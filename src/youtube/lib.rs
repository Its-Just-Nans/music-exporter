//! Youtube platform implementation
//! Useful link https://developers.google.com/youtube/v3/docs/playlistItems#resource

use reqwest::Client;

use super::types::{APIResponse, GoogleAccessToken, PlaylistItems};
use crate::{custom_env, oauth::listen_for_code, utils::input_env};

/// Youtube platform
#[derive(Default)]
pub struct YoutubePlatform {
    /// API key
    api_key: String,

    /// Authorization token
    authorization: String,
}

impl YoutubePlatform {
    /// Create a new YoutubePlatform
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Get the liked playlist id
    /// # Panics
    /// If the request fails
    async fn get_liked_playlist_id(&self) -> String {
        let authorize_url = url::Url::parse_with_params(
            "https://youtube.googleapis.com/youtube/v3/channels",
            &[
                ("part", "snippet,contentDetails,statistics"),
                ("mine", "true"),
                ("key", &self.api_key),
            ],
        )
        .unwrap();
        let resp = Client::new()
            .get(authorize_url)
            .header("Authorization", format!("Bearer {}", &self.authorization))
            .header("Accept", "application/json")
            .send()
            .await
            .unwrap();
        let json_response = match resp.status() {
            reqwest::StatusCode::OK => match resp.json::<APIResponse>().await {
                Ok(parsed) => parsed,
                Err(_) => {
                    panic!("Failed to parse response of the liked playlist");
                }
            },
            e => {
                panic!("Failed to get of the liked playlist {}", e);
            }
        };
        json_response
            .items
            .first()
            .unwrap()
            .content_details
            .related_playlists
            .likes
            .clone()
    }

    /// Get the authorization token from the code
    /// # Panics
    /// If the request fails
    async fn code_to_token(id_client: &str, id_client_secret: &str, code: &str) -> String {
        let resp = Client::new()
            .post("https://oauth2.googleapis.com/token")
            .header("Accept", "application/json")
            .form(&[
                ("code", code),
                ("client_id", id_client),
                ("client_secret", id_client_secret),
                ("redirect_uri", "http://localhost:8000"),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await;
        let json_response = resp.unwrap().json::<GoogleAccessToken>().await.unwrap();
        json_response.access_token
    }

    /// Clean the title
    fn clean_title(title: &str) -> String {
        let mut parentheses = None;
        let mut acc = None;
        let parts = title.bytes().enumerate().map(|(idx, c)| match c {
            b'(' => {
                parentheses = Some(idx);
                None
            }
            b'[' => {
                acc = Some(idx);
                None
            }
            b']' => {
                if let Some(i) = acc {
                    acc = None;
                    Some(title[i..idx + 1].to_string())
                } else {
                    None
                }
            }
            b')' => {
                if let Some(i) = parentheses {
                    parentheses = None;
                    Some(title[i..idx + 1].to_string())
                } else {
                    None
                }
            }
            _ => None,
        });
        let mut cleaned_title = title.to_string();
        for part in parts.flatten() {
            if part.to_lowercase().contains("offic") {
                let to_replace = format!(" {}", part);
                cleaned_title = cleaned_title.replace(&to_replace, "");
            }
        }
        cleaned_title
    }

    /// Get the playlist items
    /// # Panics
    /// If the request fails
    async fn get_playlist_items(
        &self,
        playlist_id: &str,
        page_token: Option<String>,
    ) -> (Vec<crate::Music>, Option<String>) {
        let url = url::Url::parse_with_params("https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet%2CcontentDetails&maxResults=50", 
        &[("playlistId", playlist_id),
        ("key", &self.api_key)]).unwrap();
        let resp = Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", &self.authorization))
            .header("Accept", "application/json")
            .query(&[("pageToken", page_token.unwrap_or_default())])
            .send()
            .await
            .unwrap();
        let json_response = match resp.status() {
            reqwest::StatusCode::OK => {
                // on success, parse our JSON to an APIResponse
                match resp.json::<PlaylistItems>().await {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        panic!("Failed to parse response of the playlist items {}", err);
                    }
                }
            }
            err => {
                panic!("Failed to get response of the playlist items {}", err);
            }
        };
        let items = json_response
            .items
            .iter()
            .map(|item| {
                let author = item
                    .snippet
                    .video_owner_channel_title
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string())
                    .replace(" - Topic", "");
                crate::Music {
                    title: Self::clean_title(&item.snippet.title),
                    author,
                    thumbnail: Some(format!(
                        "https://img.youtube.com/vi/{}/default.jpg",
                        item.snippet.resource_id.video_id
                    )),
                    url: Some(format!(
                        "https://www.youtube.com/watch?v={}",
                        item.snippet.resource_id.video_id
                    )),
                    date: Some(item.snippet.published_at.clone()),
                    album: None,
                }
            })
            .collect();
        log::info!("Next page token: {:?}", json_response.next_page_token);
        (items, json_response.next_page_token)
    }
}

impl crate::Platform for YoutubePlatform {
    async fn init() -> Result<Self, ()> {
        let api_key = input_env("Please enter API KEY", custom_env!("YOUTUBE_API_KEY"))
            .expect("API KEY is required");
        let id_client = input_env("Please enter id_client", custom_env!("YOUTUBE_ID_CLIENT"))
            .expect("ID_CLIENT is required");
        let id_client_secret = input_env(
            "Please enter id_client_secret",
            custom_env!("YOUTUBE_ID_CLIENT_SECRET"),
        )
        .expect("ID_CLIENT_SECRET is required");
        let url_oauth = format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&scope=https://www.googleapis.com/auth/youtube.readonly&response_type=code", id_client.clone(), "http://localhost:8000");
        // start the server in a thread
        let srv = listen_for_code(8000);
        println!(
            "Please go to this url to get the authorization token (or hit CTRCL+C): {}",
            url_oauth
        );
        match srv.await {
            Ok(resp) => {
                let authorization =
                    YoutubePlatform::code_to_token(&id_client, &id_client_secret, &resp.code).await;
                Ok(Self {
                    api_key,
                    authorization,
                })
            }
            Err(_) => Err(()),
        }
    }

    async fn get_list(&self) -> Vec<crate::Music> {
        let liked_playlist_id = self.get_liked_playlist_id().await;
        println!("Liked playlist id: {}", liked_playlist_id);
        let mut items = Vec::new();
        let mut page_token = None;
        loop {
            let (new_items, new_page_token) = self
                .get_playlist_items(&liked_playlist_id, page_token)
                .await;
            items.extend(new_items);
            page_token = new_page_token;
            if page_token.is_none() {
                break;
            }
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the clean_title function
    /// # Panics
    /// If the assertion fails
    #[test]
    fn test_clean_title() {
        let title = "title (feat. artist)";
        assert_eq!(YoutubePlatform::clean_title(title), "title (feat. artist)");
        let title = "title (audio officiel)";
        assert_eq!(YoutubePlatform::clean_title(title), "title");
        let title = "title [new]";
        assert_eq!(YoutubePlatform::clean_title(title), "title [new]");
        let title = "title [official video]";
        assert_eq!(YoutubePlatform::clean_title(title), "title");
    }
}
