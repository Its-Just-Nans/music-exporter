#[derive(Debug, serde::Deserialize)]
pub struct RelatedPlaylists {
    pub likes: String,
    // pub uploads: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ContentDetails {
    #[serde(rename = "relatedPlaylists")]
    pub related_playlists: RelatedPlaylists,
}

#[derive(Debug, serde::Deserialize)]
pub struct Channel {
    #[serde(rename = "contentDetails")]
    pub content_details: ContentDetails,
}

#[derive(Debug, serde::Deserialize)]
pub struct APIResponse {
    pub items: Vec<Channel>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GoogleAccessToken {
    pub access_token: String,
    // pub expires_in: i32,
    // pub scope: String,
    // pub token_type: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResourceIdPlaylistItem {
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SnippetPlaylistItem {
    pub title: String,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "videoOwnerChannelTitle")]
    pub video_owner_channel_title: Option<String>,
    #[serde(rename = "resourceId")]
    pub resource_id: ResourceIdPlaylistItem,
}

/// See https://developers.google.com/youtube/v3/docs/playlistItems#resource
#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItem {
    pub snippet: SnippetPlaylistItem,
}

#[derive(Debug, serde::Deserialize)]
pub struct PlaylistItems {
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    pub items: Vec<PlaylistItem>,
}
