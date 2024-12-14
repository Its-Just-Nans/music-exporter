use std::{env::args, path::PathBuf};

use music_exporter::{cli_main, PlatformType};

fn setup() -> PathBuf {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    let filename = if args().len() > 1 {
        args().nth(1).unwrap()
    } else {
        "data.json".to_string()
    };
    PathBuf::from(filename)
}

#[tokio::main]
async fn main() {
    println!("music-exporter");
    let filename = setup();
    cli_main(
        filename,
        None,
        &[
            PlatformType::Deezer,
            PlatformType::Spotify,
            PlatformType::Youtube,
        ],
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_main() {
        let filename = setup();
        cli_main(
            filename,
            None,
            &[
                PlatformType::Deezer,
                // PlatformType::Spotify,
                // PlatformType::Youtube,
            ],
        )
        .await;
    }
}
