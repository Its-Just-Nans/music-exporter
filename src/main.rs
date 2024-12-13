use std::{env::args, path::PathBuf};

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    println!("music-exporter");
    let filename = if args().len() > 1 {
        args().nth(1).unwrap()
    } else {
        "data.json".to_string()
    };
    let filename = PathBuf::from(filename);
    music_exporter::cli_main(filename, None).await;
}
