//! Music exporter library

#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo
)]
#![warn(clippy::multiple_crate_versions)]

mod macros;
pub(crate) mod music;
pub(crate) mod oauth;
pub(crate) mod utils;

mod deezer;
mod spotify;
mod youtube;

pub use deezer::lib::DeezerPlatform;
pub use music::Music;
pub use spotify::lib::SpotifyPlatform;
pub use youtube::lib::YoutubePlatform;

pub use utils::cli_main;
pub use utils::Platform;
pub use utils::PlatformType;

pub(crate) use macros::custom_env;
