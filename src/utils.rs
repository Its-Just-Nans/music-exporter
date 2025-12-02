//! Utility functions

use clap::{ArgAction, Parser, ValueEnum};
use serde::Serialize;
use std::{
    fs::{self, File, OpenOptions},
    future::Future,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    pin::Pin,
};

use crate::{
    errors::MusicExporterError, music, DeezerPlatform, Music, SpotifyPlatform, YoutubePlatform,
};

/// Platform trait
pub trait Platform: Send + Sync {
    /// Initialize the platform
    fn try_new(
        args: &MusicExporter,
    ) -> Pin<Box<dyn Future<Output = Result<Self, MusicExporterError>> + Send>>
    where
        Self: Sized;

    /// Get the list of music
    fn get_list<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Music>, MusicExporterError>> + Send + 'a>>;
}

/// Platform type
#[derive(Debug, Clone, ValueEnum)]
#[non_exhaustive]
pub enum PlatformType {
    /// Deezer platform
    Deezer,

    /// Spotify platform
    Spotify,

    /// Youtube platform
    Youtube,
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let platform_str = match self {
            PlatformType::Deezer => "Deezer",
            PlatformType::Spotify => "Spotify",
            PlatformType::Youtube => "Youtube",
        };
        write!(f, "{}", platform_str)
    }
}

impl PlatformType {
    /// Try to init the plateform
    /// # Errors
    /// Error if fail to init
    async fn try_init(
        &self,
        args: &MusicExporter,
    ) -> Result<Box<dyn Platform + Send + Sync>, MusicExporterError> {
        let platform: Box<dyn Platform> = match self {
            PlatformType::Deezer => Box::new(DeezerPlatform::try_new(args).await?),
            PlatformType::Spotify => Box::new(SpotifyPlatform::try_new(args).await?),
            PlatformType::Youtube => Box::new(YoutubePlatform::try_new(args).await?),
        };
        Ok(platform)
    }
}

/// Music-exporter args
#[derive(Debug, Parser)]
#[command(
    name = "music-exporter",
    about = "Exports music files for given platforms"
)]
pub struct MusicExporter {
    /// Path to optional .env file
    #[arg(long, value_name = "ENV_FILE", required = false)]
    pub env_file: Option<PathBuf>,

    /// Path to the music file
    #[arg(long, value_name = "MUSIC_FILE", required = true)]
    pub music_file: PathBuf,

    /// Remove duplicates
    #[arg(long, action=ArgAction::SetFalse)]
    pub remove_duplicates: bool,

    /// Sort musics
    #[arg(long, action=ArgAction::SetFalse)]
    pub sort: bool,

    /// Target platforms (must provide at least one)
    #[arg(long = "platform", value_enum, required = true, num_args = 1..)]
    pub platforms: Vec<PlatformType>,

    /// Custom youtube playlist id
    #[arg(long, visible_alias = "ytb-playlist-id")]
    pub youtube_playlist_id: Option<String>,
}

/// Main function for the CLI
/// # Errors
/// Fails on error
pub async fn music_exporter_main(
    music_file: PathBuf,
    env_path: Option<PathBuf>,
    platforms: &[PlatformType],
) -> Result<Vec<Music>, MusicExporterError> {
    let m = MusicExporter::new_from_vars(music_file.clone(), env_path, platforms);
    m.run_main().await
}

impl MusicExporter {
    /// Create new from args
    pub fn new_from_vars(
        music_file: PathBuf,
        env_path: Option<PathBuf>,
        platforms: &[PlatformType],
    ) -> Self {
        Self {
            music_file,
            env_file: env_path,
            platforms: platforms.to_vec(),
            youtube_playlist_id: None,
            remove_duplicates: true,
            sort: true,
        }
    }

    /// Get all the musics
    /// # Errors
    /// Fails on request
    pub async fn get_musics(&self) -> Result<Vec<Music>, MusicExporterError> {
        let mut items = self.read_from_file()?;
        let musics_from_platforms = self.get_music_from_platforms().await?;
        items.extend(musics_from_platforms);
        // write to file
        log::info!("Total items: {}", items.len());
        let mut items = if self.remove_duplicates {
            music::unique_music(items)
        } else {
            items
        };
        let items = if self.sort {
            items.sort();
            items
        } else {
            items
        };
        Ok(items)
    }

    /// Load the env file
    /// # Errors
    /// Fails if the env failed to load
    pub fn load_env(&self) -> Result<(), MusicExporterError> {
        match &self.env_file {
            Some(path) => {
                dotenv::from_path(path)?;
            }
            None => {
                // dotenv::dotenv()?;
            }
        };
        Ok(())
    }

    /// Run main
    /// # Errors
    /// Fails if something happens
    pub async fn run_main(&self) -> Result<Vec<Music>, MusicExporterError> {
        self.load_env()
            .map_err(|e| MusicExporterError::new_with_source("Failed to load env file", e))?;
        let musics = self.get_musics().await?;
        log::info!("Unique items: {}", musics.len());
        self.write_to_file(&musics)?;
        Ok(musics)
    }

    /// Get the list of music from the selected platforms
    /// # Errors
    /// Fails if fail to get lists
    pub async fn get_music_from_platforms(&self) -> Result<Vec<Music>, MusicExporterError> {
        let mut items = vec![];
        for platform_type in &self.platforms {
            log::info!("Retrieving music of {}", platform_type);
            let plateform = platform_type.try_init(self).await?;
            let musics = plateform.get_list().await?;
            items.extend(musics);
        }
        Ok(items)
    }

    /// Write to file
    /// # Errors
    /// Error if the file cannot be created
    pub fn write_to_file(&self, data: &[Music]) -> Result<(), MusicExporterError> {
        let file = File::create(&self.music_file)?;
        let mut writer = BufWriter::new(file);
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
        data.serialize(&mut ser)?;
        writer.flush()?;
        Ok(())
    }

    /// Read from file
    /// # Errors
    /// Error if the file cannot be created
    pub fn read_from_file(&self) -> Result<Vec<Music>, MusicExporterError> {
        if !self.music_file.exists() {
            if let Some(parent) = self.music_file.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&self.music_file)?;
            file.write_all(b"[]")?;
        }
        let file = File::open(&self.music_file)?;
        let reader = BufReader::new(file);
        let mut de = serde_json::Deserializer::from_reader(reader);
        let items = serde::de::Deserialize::deserialize(&mut de).unwrap_or(vec![]);
        Ok(items)
    }

    /// Get all the platform types
    pub fn get_all_platform_type() -> Vec<PlatformType> {
        vec![
            PlatformType::Deezer,
            PlatformType::Spotify,
            PlatformType::Youtube,
        ]
    }
}

/// Input from the environment
/// # Errors
/// Error if the input is not correct
pub fn input_env(txt: &str, env_name: &str) -> Result<String, MusicExporterError> {
    use std::io::{stdin, stdout, Write};
    if let Ok(val) = std::env::var(env_name) {
        return Ok(val);
    }
    let mut s = String::new();
    print!("{}", txt);
    println!(" ({} not found in the env)", env_name);
    let _ = stdout().flush();
    stdin().read_line(&mut s)?;
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}

/// Convert to base64
pub fn to_base_64(input: &str) -> String {
    use base64::Engine;
    let mut output = String::new();
    base64::prelude::BASE64_STANDARD.encode_string(input, &mut output);
    output
}
