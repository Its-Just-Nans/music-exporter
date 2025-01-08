//! Utility functions

use crate::{music, DeezerPlatform, Music, SpotifyPlatform, YoutubePlatform};
use std::path::PathBuf;

/// Platform trait
pub trait Platform {
    /// Initialize the platform
    fn init() -> impl std::future::Future<Output = Result<Self, ()>> + Send
    where
        Self: Sized;

    /// Get the list of music
    fn get_list(&self) -> impl std::future::Future<Output = Vec<Music>> + Send;
}

/// Platform type
pub enum PlatformType {
    /// Deezer platform
    Deezer,

    /// Spotify platform
    Spotify,

    /// Youtube platform
    Youtube,
}

/// Get the list of music from the selected platforms
pub async fn get_list(platforms: &[PlatformType]) -> Vec<Music> {
    let mut items = vec![];
    for platform in platforms {
        match platform {
            PlatformType::Deezer => {
                if let Ok(deezer) = DeezerPlatform::init().await {
                    items.extend(deezer.get_list().await);
                }
            }
            PlatformType::Spotify => {
                if let Ok(spt) = SpotifyPlatform::init().await {
                    items.extend(spt.get_list().await);
                }
            }
            PlatformType::Youtube => {
                if let Ok(ytb) = YoutubePlatform::init().await {
                    items.extend(ytb.get_list().await);
                }
            }
        }
    }
    items
}

/// Main function for the CLI
pub async fn cli_main(music_file: PathBuf, env_path: Option<PathBuf>, platforms: &[PlatformType]) {
    match env_path {
        Some(path) => {
            dotenv::from_path(path).ok();
        }
        None => {
            dotenv::dotenv().ok();
        }
    }
    let mut items = vec![];
    read_from_file(&music_file, &mut items);

    if platforms.is_empty() {
        log::info!("No platform selected, using all platforms");
    }
    items.extend(get_list(platforms).await);
    // write to file
    log::info!("Total items: {}", items.len());
    let mut items = music::unique_music(items);
    items.sort();
    log::info!("Unique items: {}", items.len());
    write_to_file(&music_file, items);
}

/// Input from the environment
/// # Panics
/// Panics if the input is not correct
pub fn input_env(txt: &str, env_name: &str) -> Option<String> {
    use std::io::{stdin, stdout, Write};
    if let Ok(val) = std::env::var(env_name) {
        return Some(val);
    }
    let mut s = String::new();
    print!("{}", txt);
    println!(" ({} not found in the env)", env_name);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Some(s)
}

/// Write to file
/// # Panics
/// Panics if the file cannot be created
pub fn write_to_file(filename: &PathBuf, data: Vec<crate::Music>) {
    use serde::Serialize;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    let file = File::create(filename).expect("Could not create file");
    let mut writer = BufWriter::new(file);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
    data.serialize(&mut ser).unwrap();
    writer.flush().unwrap();
}

/// Read from file
/// # Panics
/// Panics if the file cannot be created
pub fn read_from_file(filename: &PathBuf, items: &mut Vec<crate::Music>) {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(filename).expect("Could not create file");
    let reader = BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    items.clear();
    items.extend(serde::de::Deserialize::deserialize(&mut de).unwrap_or(vec![]));
}

/// Convert to base64
pub fn to_base_64(input: &str) -> String {
    use base64::Engine;
    let mut output = String::new();
    base64::prelude::BASE64_STANDARD.encode_string(input, &mut output);
    output
}
