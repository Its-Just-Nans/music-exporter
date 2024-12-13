//! Music exporter library

pub mod music;
mod oauth;
mod spotify;
pub mod utils;
mod youtube;
pub use music::Music;
pub use spotify::spotify::SpotifyPlatform;
pub use utils::input;
pub use youtube::youtube::YoutubePlatform;

pub trait Platform {
    fn init(self) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_list(&self) -> impl std::future::Future<Output = Vec<Music>> + Send;
}

use std::path::PathBuf;

use utils::{read_from_file, write_to_file};

pub async fn cli_main(music_file: PathBuf, env_path: Option<PathBuf>) {
    dotenv::from_path(env_path.unwrap_or_else(|| PathBuf::from(".env"))).ok();
    let mut items = vec![];
    read_from_file(&music_file, &mut items);
    let ytb = YoutubePlatform::new().init().await;
    items.extend(ytb.get_list().await);

    let spt = SpotifyPlatform::new().init().await;
    items.extend(spt.get_list().await);
    // write to file
    log::info!("Total items: {}", items.len());
    items = music::unique_music(items);
    log::info!("Unique items: {}", items.len());
    items.sort();
    write_to_file(&music_file, items);
}
