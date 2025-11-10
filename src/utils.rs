//! Utility functions

use clap::{Parser, ValueEnum};
use std::{
    fs::{self, File, OpenOptions},
    future::Future,
    io::{BufReader, Write},
    path::PathBuf,
    pin::Pin,
};

use crate::{
    errors::MusicExporterError, music, DeezerPlatform, Music, SpotifyPlatform, YoutubePlatform,
};

/// Platform trait
pub trait Platform: Send + Sync {
    /// Initialize the platform
    fn try_new() -> Pin<Box<dyn Future<Output = Result<Self, MusicExporterError>> + Send>>
    where
        Self: Sized;

    /// Get the list of music
    fn get_list<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Music>, MusicExporterError>> + Send + 'a>>;
}

/// Platform type
#[derive(Debug, Clone, ValueEnum)]
pub enum PlatformType {
    /// Deezer platform
    Deezer,

    /// Spotify platform
    Spotify,

    /// Youtube platform
    Youtube,
}

impl PlatformType {
    /// Try to init the plateform
    /// # Errors
    /// Error if fail to init
    async fn try_init(&self) -> Result<Box<dyn Platform + Send + Sync>, MusicExporterError> {
        let platform: Box<dyn Platform> = match self {
            PlatformType::Deezer => Box::new(DeezerPlatform::try_new().await?),
            PlatformType::Spotify => Box::new(SpotifyPlatform::try_new().await?),
            PlatformType::Youtube => Box::new(YoutubePlatform::try_new().await?),
        };
        Ok(platform)
    }

    /// Try to get the list for the plateform
    /// # Errors
    /// Error if fail to get the list
    async fn get_list(&self) -> Result<Vec<Music>, MusicExporterError> {
        let plateform: Box<dyn Platform + Send + Sync> = self.try_init().await?;
        plateform.get_list().await
    }
}

/// Get the list of music from the selected platforms
/// # Errors
/// Fails if fail to get lists
pub async fn get_list(platforms: &[PlatformType]) -> Result<Vec<Music>, MusicExporterError> {
    let mut items = vec![];
    for platform in platforms {
        items.extend(platform.get_list().await?);
    }
    Ok(items)
}

/// Music-exporter args
#[derive(Debug, Parser)]
#[command(
    name = "music-exporter",
    about = "Exports music files for given platforms"
)]
pub struct MusicExporterArgs {
    /// Path to optional .env file
    #[arg(long, value_name = "ENV_FILE", required = false)]
    pub env_file: Option<PathBuf>,

    /// Path to the music file
    #[arg(long, value_name = "MUSIC_FILE", required = true)]
    pub music_file: PathBuf,

    /// Target platforms (must provide at least one)
    #[arg(long = "platform", value_enum, required = true, num_args = 1..)]
    pub platforms: Vec<PlatformType>,
}

/// Main function for the CLI
/// # Errors
/// Fails on error
pub async fn music_exporter_main(
    music_file: PathBuf,
    env_path: Option<PathBuf>,
    platforms: &[PlatformType],
) -> Result<(), MusicExporterError> {
    match env_path {
        Some(path) => {
            dotenv::from_path(path).ok();
        }
        None => {
            dotenv::dotenv().ok();
        }
    }
    let mut items = read_from_file(&music_file)?;

    if platforms.is_empty() {
        log::info!("No platform selected, using all platforms");
    }
    items.extend(get_list(platforms).await?);
    // write to file
    log::info!("Total items: {}", items.len());
    let mut items = music::unique_music(items);
    items.sort();
    log::info!("Unique items: {}", items.len());
    write_to_file(&music_file, items)?;
    Ok(())
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

/// Write to file
/// # Errors
/// Error if the file cannot be created
pub fn write_to_file(
    filename: &PathBuf,
    data: Vec<crate::Music>,
) -> Result<(), MusicExporterError> {
    use serde::Serialize;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    let file = File::create(filename)?;
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
pub fn read_from_file(filename: &PathBuf) -> Result<Vec<Music>, MusicExporterError> {
    if !filename.exists() {
        if let Some(parent) = filename.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;
        file.write_all(b"[]")?; // write bytes
    }
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    let items = serde::de::Deserialize::deserialize(&mut de).unwrap_or(vec![]);
    Ok(items)
}

/// Convert to base64
pub fn to_base_64(input: &str) -> String {
    use base64::Engine;
    let mut output = String::new();
    base64::prelude::BASE64_STANDARD.encode_string(input, &mut output);
    output
}
