//! Spotify API types

/// Spotify Artist
#[derive(Debug, serde::Deserialize)]
pub struct SpotifyArtist {
    /// Artist name
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyImage {
    /// Image url
    pub url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyAlbum {
    /// Album release date
    pub release_date: String,
    /// Album images
    pub images: Vec<SpotifyImage>,
    /// Album name
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TrackPlaylistItem {
    /// Track name
    pub name: String,

    /// Track album
    pub album: SpotifyAlbum,

    /// Track artists
    pub artists: Vec<SpotifyArtist>,

    /// Track external urls
    pub external_urls: SpotifyExternalUrls,
}
#[derive(Debug, serde::Deserialize)]
pub struct SpotifyExternalUrls {
    /// Spotify url
    pub spotify: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItem {
    /// Track item
    pub track: TrackPlaylistItem,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItems {
    /// Next page url
    pub next: Option<String>,

    /// Offset
    pub offset: u64,

    /// Playlist items
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpotifyAccessToken {
    /// Access token
    pub access_token: String,
}
