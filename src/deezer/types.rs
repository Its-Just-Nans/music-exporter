use std::str;

#[derive(Debug, serde::Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Album {
    pub title: String,
    pub cover: String,
    // pub cover_small: String,
    // pub cover_medium: String,
    // pub cover_big: String,
    // pub cover_xl: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TrackItem {
    // pub id: i64,
    // pub readable: bool,
    pub title: String,
    pub link: String,
    // pub duration: u64,
    // pub rank: u64,
    // pub explicit_lyrics: bool,
    // pub time_add: u64,
    pub album: Album,
    pub artist: Artist,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse {
    pub next: Option<String>,
    // pub total: u64,
    pub data: Vec<TrackItem>,
}
