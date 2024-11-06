mod oauth;
mod spotify;
mod youtube;

pub fn input(txt: &str, env_name: &str) -> Option<String> {
    use std::io::{stdin, stdout, Write};
    if let Ok(val) = std::env::var(env_name) {
        return Some(val);
    }
    let mut s = String::new();
    print!("{}", txt);
    println!("({} not found in the env)", env_name);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Some(s)
}

pub fn write_to_file(filename: &str, data: Vec<Music>) {
    use serde::Serialize;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    let file = File::create(filename).expect("Could not create file");
    let mut writer = BufWriter::new(file);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
    data.serialize(&mut ser).unwrap();
    writer.flush().unwrap();
}

pub trait Platform {
    fn init(&mut self) -> impl std::future::Future<Output = ()> + Send;
    fn get_list(&self) -> impl std::future::Future<Output = Vec<crate::Music>> + Send;
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Music {
    pub title: String,
    pub author: String,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
    pub date: Option<String>,
}

pub use spotify::spotify::SpotifyPlatform;
pub use youtube::youtube::YoutubePlatform;
