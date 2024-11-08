//!
//! Useful link https://developers.google.com/youtube/v3/docs/playlistItems#resource

use reqwest::Client;

use super::types::{APIResponse, GoogleAccessToken, PlaylistItems};
use crate::{input, oauth::listen_for_code};

pub struct YoutubePlatform {
    api_key: Option<String>,
    authorization: Option<String>,
    id_client: Option<String>,
    id_client_secret: Option<String>,
}

impl YoutubePlatform {
    pub fn new() -> Self {
        Self {
            api_key: None,
            authorization: None,
            id_client: None,
            id_client_secret: None,
        }
    }
    async fn get_liked_playlist_id(&self) -> String {
        let authorize_url = url::Url::parse_with_params(
            "https://youtube.googleapis.com/youtube/v3/channels",
            &[
                ("part", "snippet,contentDetails,statistics"),
                ("mine", "true"),
                ("key", self.api_key.as_ref().unwrap()),
            ],
        )
        .unwrap();
        let resp = Client::new()
            .get(authorize_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization.as_ref().unwrap()),
            )
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
        return json_response
            .items
            .first()
            .unwrap()
            .content_details
            .related_playlists
            .likes
            .clone();
    }

    async fn code_to_token(&self, code: &str) -> String {
        let resp = Client::new()
            .post("https://oauth2.googleapis.com/token")
            .header("Accept", "application/json")
            .form(&[
                ("code", code),
                ("client_id", self.id_client.as_ref().unwrap()),
                ("client_secret", self.id_client_secret.as_ref().unwrap()),
                ("redirect_uri", "http://localhost:8000"),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await;
        let json_response = resp.unwrap().json::<GoogleAccessToken>().await.unwrap();
        return json_response.access_token;
    }

    async fn get_playlist_items(
        &self,
        playlist_id: &str,
        page_token: Option<String>,
    ) -> (Vec<crate::Music>, Option<String>) {
        let url = url::Url::parse_with_params("https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet%2CcontentDetails&maxResults=50", 
        &[("playlistId", playlist_id),
        ("key", self.api_key.as_ref().unwrap())]).unwrap();
        let resp = Client::new()
            .get(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization.as_ref().unwrap()),
            )
            .header("Accept", "application/json")
            .query(&[("pageToken", page_token.unwrap_or_else(|| "".to_string()))])
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
                return crate::Music {
                    title: item.snippet.title.clone(),
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
                };
            })
            .collect();
        log::info!("Next page token: {:?}", json_response.next_page_token);
        (items, json_response.next_page_token)
    }
}

impl crate::Platform for YoutubePlatform {
    async fn init(&mut self) {
        self.api_key = input("Please enter API KEY", "MUSIC_EXPLORER_YOUTUBE_API_KEY");
        self.id_client = input("Please enter id_client", "MUSIC_EXPLORER_YOUTUBE_ID_CLIENT");
        self.id_client_secret = input(
            "Please enter id_client_secret",
            "MUSIC_EXPLORER_YOUTUBE_ID_CLIENT_SECRET",
        );
        let url_oauth = format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&scope=https://www.googleapis.com/auth/youtube.readonly&response_type=code", self.id_client.clone().unwrap(), "http://localhost:8000");
        // start the server in a thread
        let srv = listen_for_code(8000);
        println!(
            "Please go to this url to get the authorization token: {}",
            url_oauth
        );
        let resp = srv.await.unwrap();
        self.authorization = Some(self.code_to_token(&resp.code).await);
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
