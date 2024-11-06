use dotenv::dotenv;
use music_exporter::write_to_file;
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
    write_to_file("data.json", items.clone()); // check write to file
    let mut ytb = YoutubePlatform::new();
    ytb.init().await;
    items.extend(ytb.get_list().await);

    let mut spt = SpotifyPlatform::new();
    spt.init().await;
    items.extend(spt.get_list().await);
    // write to file
    write_to_file("data.json", items);
}
