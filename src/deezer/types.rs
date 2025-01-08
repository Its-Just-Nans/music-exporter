//! Types for the Deezer API

/// Artist struct
#[derive(Debug, serde::Deserialize)]
pub struct Artist {
    /// Artist name
    pub name: String,
}

/// Album struct
#[derive(Debug, serde::Deserialize)]
pub struct Album {
    /// Album title
    pub title: String,
    /// Album cover
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
    /// Track title
    pub title: String,

    /// Track link
    pub link: String,
    // pub duration: u64,
    // pub rank: u64,
    // pub explicit_lyrics: bool,
    // pub time_add: u64,
    /// Album struct
    pub album: Album,

    /// Artist struct
    pub artist: Artist,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse {
    /// Next page URL
    pub next: Option<String>,
    // pub total: u64,
    /// Track items
    pub data: Vec<TrackItem>,
}
