//! Music exporter
//!
//! # CLI Usage
//!
//! ```sh
//! cargo install music-exporter
//! music-exporter --music-file musics.json --platform deezer
//! ```
//!
//! # Rust Usage
//!
//! ```rust
//! use music_exporter::{Music, MusicExporter, MusicExporterError};
//! use clap::Parser;
//!
//! async fn run_async() -> Result<Vec<Music>, MusicExporterError> {
//!    let music_exp = MusicExporter::parse();
//!    // music_exp.load_env()?;
//!    music_exp.get_musics().await
//! }
//! ```
//!

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
mod errors;
mod spotify;
mod youtube;

pub use deezer::lib::DeezerPlatform;
pub use music::Music;
pub use spotify::lib::SpotifyPlatform;
pub use youtube::lib::YoutubePlatform;

pub use errors::MusicExporterError;
pub use utils::music_exporter_main;
pub use utils::MusicExporter;
pub use utils::Platform;
pub use utils::PlatformType;

pub(crate) use macros::custom_env;
