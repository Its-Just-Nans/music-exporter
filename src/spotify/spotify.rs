//!
//! Useful link https://developer.spotify.com/documentation/web-api

use base64::Engine;
use reqwest::Client;

use super::types::{PlaylistItems, SpotifyAccessToken};
use crate::{input, oauth::listen_for_code};

pub struct SpotifyPlatform {
    authorization: Option<String>,
    id_client: Option<String>,
    id_client_secret: Option<String>,
}

fn to_base_64(input: &str) -> String {
    let mut output = String::new();
    base64::prelude::BASE64_STANDARD.encode_string(input, &mut output);
    output
}

impl SpotifyPlatform {
    pub fn new() -> Self {
        Self {
            authorization: None,
            id_client: None,
            id_client_secret: None,
        }
    }

    async fn code_to_token(&self, code: &str) -> String {
        let authorization_header = format!(
            "Basic {}",
            to_base_64(&format!(
                "{}:{}",
                self.id_client.clone().unwrap(),
                self.id_client_secret.clone().unwrap()
            ))
        );
        let resp = Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", authorization_header)
            .form(&[
                ("code", code),
                ("redirect_uri", "http://localhost:8000"),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .unwrap();
        let json_response = resp.json::<SpotifyAccessToken>().await.unwrap();
        return json_response.access_token;
    }

    async fn get_playlist_items(&self, offset: Option<u64>) -> (Vec<crate::Music>, Option<u64>) {
        let url = url::Url::parse_with_params(
            "https://api.spotify.com/v1/me/tracks",
            &[
                ("limit", 50.to_string()), // 50 is the maximum
                ("offset", offset.unwrap_or(0).to_string()),
            ],
        )
        .unwrap();
        let resp = Client::new()
            .get(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization.as_ref().unwrap()),
            )
            .header("Accept", "application/json")
            .send()
            .await
            .unwrap();
        let json_response = match resp.status() {
            reqwest::StatusCode::OK => {
                // on success, parse our JSON to an APIResponse
                match resp.json::<PlaylistItems>().await {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        panic!("Failed to parse response {}", err);
                    }
                }
            }
            err => {
                panic!("Failed to get response for the playlist items {}", err);
            }
        };
        let items = json_response
            .items
            .iter()
            .map(|item| crate::Music {
                title: item.track.name.clone(),
                author: item.track.artists[0].name.clone(),
                thumbnail: Some(item.track.album.images[0].url.clone()),
                url: Some(item.track.external_urls.spotify.clone()),
                date: Some(item.track.album.release_date.clone()),
            })
            .collect();
        let current_offset = json_response.offset;
        let next_offset = if json_response.next.is_some() {
            Some(current_offset + 50)
        } else {
            None
        };
        log::info!("Next offset: {:?}", next_offset);
        (items, next_offset)
    }
}

impl crate::Platform for SpotifyPlatform {
    async fn init(&mut self) {
        self.id_client = input("Please enter id_client", "MUSIC_EXPLORER_SPOTIFY_ID_CLIENT");
        self.id_client_secret = input(
            "Please enter id_client_secret",
            "MUSIC_EXPLORER_SPOTIFY_ID_CLIENT_SECRET",
        );
        let url_oauth = url::Url::parse_with_params(
            "https://accounts.spotify.com/authorize",
            &[
                ("client_id", self.id_client.clone().unwrap()),
                ("response_type", "code".to_string()),
                ("redirect_uri", "http://localhost:8000".to_string()),
                (
                    "scope",
                    "playlist-read-private,user-library-read".to_string(),
                ),
            ],
        )
        .unwrap();
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
        let mut items = Vec::new();
        let mut page_next = None;
        loop {
            let (new_items, next_page) = self.get_playlist_items(page_next).await;
            items.extend(new_items);
            page_next = next_page;
            if page_next.is_none() {
                break;
            }
        }
        items
    }
}
