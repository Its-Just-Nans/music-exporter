use clap::Parser;
use std::process::exit;

use music_exporter::MusicExporter;

#[tokio::main]
async fn main() {
    println!("music-exporter");
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    let music_exp = MusicExporter::parse();
    match music_exp.run_main().await {
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
        MusicExporter::new_from_vars(
            filename,
            None,
            &[
                PlatformType::Deezer,
                // PlatformType::Spotify,
                // PlatformType::Youtube,
            ],
        )
        .run_main()
        .await
        .unwrap();
    }
}
