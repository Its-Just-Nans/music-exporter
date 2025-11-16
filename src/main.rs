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