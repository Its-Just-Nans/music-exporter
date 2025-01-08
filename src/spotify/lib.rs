//! Spotify platform implementation
//! Useful link https://developer.spotify.com/documentation/web-api

use reqwest::Client;

use super::types::{PlaylistItems, SpotifyAccessToken};
use crate::{
    custom_env,
    oauth::listen_for_code,
    utils::{input_env, to_base_64},
};

/// Spotify platform
#[derive(Default)]
pub struct SpotifyPlatform {
    /// Authorization token
    authorization: String,
}

impl SpotifyPlatform {
    /// Get the authorization token from the code
    /// # Panics
    /// If the request fails
    async fn code_to_token(id_client: &str, id_client_secret: &str, code: &str) -> String {
        let authorization_header = format!(
            "Basic {}",
            to_base_64(&format!("{}:{}", id_client, id_client_secret))
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
        json_response.access_token
    }

    /// Get the playlist items
    /// # Panics
    /// If the request fails
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
            .header("Authorization", format!("Bearer {}", &self.authorization))
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
                album: Some(item.track.album.name.clone()),
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
    async fn init() -> Result<Self, ()> {
        let id_client = input_env("Please enter id_client", custom_env!("SPOTIFY_ID_CLIENT"))
            .expect("ID_CLIENT is required");
        let id_client_secret = input_env(
            "Please enter id_client_secret",
            custom_env!("SPOTIFY_ID_CLIENT_SECRET"),
        )
        .expect("ID_CLIENT_SECRET is required");
        let url_oauth = url::Url::parse_with_params(
            "https://accounts.spotify.com/authorize",
            &[
                ("client_id", &id_client),
                ("response_type", &"code".to_string()),
                ("redirect_uri", &"http://localhost:8000".to_string()),
                (
                    "scope",
                    &"playlist-read-private,user-library-read".to_string(),
                ),
            ],
        )
        .unwrap();
        // start the server in a thread
        let srv = listen_for_code(8000);
        println!(
            "Please go to this url to get the authorization token (or hit CTRCL+C): {}",
            url_oauth
        );
        match srv.await {
            Ok(resp) => {
                let authorization =
                    SpotifyPlatform::code_to_token(&id_client, &id_client_secret, &resp.code).await;
                Ok(Self { authorization })
            }
            Err(_) => Err(()),
        }
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
