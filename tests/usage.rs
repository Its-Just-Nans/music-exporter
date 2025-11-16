#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;
    use music_exporter::{MusicExporter, PlatformType};

    #[tokio::test]
    async fn parse_failure() {
        let music_exp = MusicExporter::try_parse();
        assert!(music_exp.is_err());
    }

    #[tokio::test]
    async fn build_from_args() {
        let music_exp = MusicExporter::new_from_vars(PathBuf::from("unknown.json"), None, &[]);
        let musics = music_exp.get_musics().await.unwrap();
        assert_eq!(musics, vec![]);
    }

    #[tokio::test]
    async fn test_main() {
        let filename = PathBuf::from("data.json");
        let main_res = MusicExporter::new_from_vars(
            filename,
            Some(PathBuf::from(".env.fake")),
            &[
                PlatformType::Deezer,
                // PlatformType::Spotify,
                // PlatformType::Youtube,
            ],
        )
        .run_main()
        .await;
        assert!(main_res.is_err()); // env not found
        let err = main_res.unwrap_err();
        assert_eq!(err.message, "Failed to load env file");
    }
}
