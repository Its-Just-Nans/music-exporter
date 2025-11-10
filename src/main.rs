use clap::Parser;
use std::process::exit;

use music_exporter::{music_exporter_main, MusicExporterArgs};

#[tokio::main]
async fn main() {
    println!("music-exporter");
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    let args = MusicExporterArgs::parse();
    match music_exporter_main(args.music_file, args.env_file, &args.platforms).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use music_exporter::PlatformType;

    use super::*;

    #[tokio::test]
    async fn test_main() {
        let filename = PathBuf::from("data.json");
        music_exporter_main(
            filename,
            None,
            &[
                PlatformType::Deezer,
                // PlatformType::Spotify,
                // PlatformType::Youtube,
            ],
        )
        .await
        .unwrap();
    }
}
