use std::str;

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyArtist {
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyImage {
    pub url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyAlbum {
    pub release_date: String,
    pub images: Vec<SpotifyImage>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TrackPlaylistItem {
    pub name: String,
    pub album: SpotifyAlbum,
    pub artists: Vec<SpotifyArtist>,
    pub external_urls: SpotifyExternalUrls,
}
#[derive(Debug, serde::Deserialize)]
pub struct SpotifyExternalUrls {
    pub spotify: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItem {
    pub track: TrackPlaylistItem,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItems {
    pub next: Option<String>,
    pub offset: u64,
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyAccessToken {
    pub access_token: String,
}
