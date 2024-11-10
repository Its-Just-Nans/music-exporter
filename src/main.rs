use std::env::args;

use dotenv::dotenv;
use music_exporter::utils::{read_from_file, write_to_file};
use music_exporter::Platform;
use music_exporter::SpotifyPlatform;
use music_exporter::YoutubePlatform;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    println!("music-exporter");
    dotenv().ok();
    let mut items = vec![];
    let filename = if args().len() > 1 {
        args().nth(1).unwrap()
    } else {
        "data.json".to_string()
    };
    read_from_file(&filename, &mut items);
    let ytb = YoutubePlatform::new().init().await;
    items.extend(ytb.get_list().await);

    let spt = SpotifyPlatform::new().init().await;
    items.extend(spt.get_list().await);
    // write to file
    log::info!("Total items: {}", items.len());
    items = music_exporter::music::unique_music(items);
    log::info!("Unique items: {}", items.len());
    items.sort();
    write_to_file(&filename, items);
}
