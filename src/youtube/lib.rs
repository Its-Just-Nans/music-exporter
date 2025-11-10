//! Youtube platform implementation
//! Useful link https://developers.google.com/youtube/v3/docs/playlistItems#resource

use std::{future::Future, pin::Pin};

use reqwest::Client;

use super::types::{APIResponse, GoogleAccessToken, PlaylistItems};
use crate::{
    custom_env, errors::MusicExporterError, oauth::listen_for_code, utils::input_env, Music,
};

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
    /// # Errors
    /// If the request fails
    async fn get_liked_playlist_id(&self) -> Result<String, MusicExporterError> {
        let authorize_url = url::Url::parse_with_params(
            "https://youtube.googleapis.com/youtube/v3/channels",
            &[
                ("part", "snippet,contentDetails,statistics"),
                ("mine", "true"),
                ("key", &self.api_key),
            ],
        )?;
        let resp = Client::new()
            .get(authorize_url)
            .header("Authorization", format!("Bearer {}", &self.authorization))
            .header("Accept", "application/json")
            .send()
            .await?;
        let json_response = match resp.status() {
            reqwest::StatusCode::OK => resp.json::<APIResponse>().await?,
            err => {
                return Err(MusicExporterError::new(format!(
                    "Failed to get of the liked playlist {}",
                    err
                )))
            }
        };
        match json_response.items.first() {
            Some(first_playlist) => Ok(first_playlist
                .content_details
                .related_playlists
                .likes
                .clone()),
            None => Err(MusicExporterError::new("Cannot get liked playlist")),
        }
    }

    /// Get the authorization token from the code
    /// # Errors
    /// If the request fails
    async fn code_to_token(
        id_client: &str,
        id_client_secret: &str,
        code: &str,
    ) -> Result<String, MusicExporterError> {
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
            .await?;
        let json_response = resp.json::<GoogleAccessToken>().await?;
        Ok(json_response.access_token)
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
    /// # Errors
    /// If the request fails
    async fn get_playlist_items(
        &self,
        playlist_id: &str,
        page_token: Option<String>,
    ) -> Result<(Vec<crate::Music>, Option<String>), MusicExporterError> {
        let url = url::Url::parse_with_params("https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet%2CcontentDetails&maxResults=50", 
        &[("playlistId", playlist_id),
        ("key", &self.api_key)])?;
        let resp = Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", &self.authorization))
            .header("Accept", "application/json")
            .query(&[("pageToken", page_token.unwrap_or_default())])
            .send()
            .await?;
        let json_response = match resp.status() {
            reqwest::StatusCode::OK => {
                // on success, parse our JSON to an APIResponse
                resp.json::<PlaylistItems>().await?
            }
            err => {
                return Err(MusicExporterError::new(format!(
                    "Failed to get response for the playlist items {}",
                    err
                )))
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
        Ok((items, json_response.next_page_token))
    }
}

impl crate::Platform for YoutubePlatform {
    fn try_new() -> Pin<Box<dyn Future<Output = Result<Self, MusicExporterError>> + Send>> {
        Box::pin(async {
            let api_key = input_env("Please enter API KEY", custom_env!("YOUTUBE_API_KEY"))?;
            let id_client = input_env("Please enter id_client", custom_env!("YOUTUBE_ID_CLIENT"))?;
            let id_client_secret = input_env(
                "Please enter id_client_secret",
                custom_env!("YOUTUBE_ID_CLIENT_SECRET"),
            )?;
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
                        YoutubePlatform::code_to_token(&id_client, &id_client_secret, &resp.code)
                            .await?;
                    Ok(Self {
                        api_key,
                        authorization,
                    })
                }
                Err(_) => Err(MusicExporterError::new("Failed to get the code")),
            }
        })
    }

    fn get_list<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Music>, MusicExporterError>> + Send + 'a>> {
        Box::pin(async {
            let liked_playlist_id = self.get_liked_playlist_id().await?;
            println!("Liked playlist id: {}", liked_playlist_id);
            let mut items = Vec::new();
            let mut page_token = None;
            loop {
                let (new_items, new_page_token) = self
                    .get_playlist_items(&liked_playlist_id, page_token)
                    .await?;
                items.extend(new_items);
                page_token = new_page_token;
                if page_token.is_none() {
                    break;
                }
            }
            Ok(items)
        })
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
