//! Music exporter library

// providers
mod deezer;
mod spotify;
mod youtube;
pub use deezer::lib::DeezerPlatform;
pub use spotify::lib::SpotifyPlatform;
pub use youtube::lib::YoutubePlatform;

pub mod music;
pub use music::Music;
mod oauth;

pub mod utils;
pub use utils::cli_main;
pub use utils::PlatformType;

/// Platform trait to implement for each platform
pub use utils::Platform;
