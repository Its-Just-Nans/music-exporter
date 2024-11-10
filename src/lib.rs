//! Music exporter library

pub mod music;
mod oauth;
mod spotify;
pub mod utils;
mod youtube;

pub trait Platform {
    fn init(self) -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn get_list(&self) -> impl std::future::Future<Output = Vec<Music>> + Send;
}

pub use music::Music;
pub use spotify::spotify::SpotifyPlatform;
pub use utils::input;
pub use youtube::youtube::YoutubePlatform;
