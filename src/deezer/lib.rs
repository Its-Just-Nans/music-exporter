//!
//! Useful link https://developers.deezer.com/api

use reqwest::Client;

use super::types::ApiResponse;
use crate::utils::input;

#[derive(Default)]
pub struct DeezerPlatform {
    cookie: String,
    user_id: String,
}

impl DeezerPlatform {
    async fn get_playlist_items(&self, offset: Option<u64>) -> (Vec<crate::Music>, Option<u64>) {
        let url = url::Url::parse_with_params(
            &format!("https://api.deezer.com/user/{}/tracks", self.user_id),
            &[
                ("index", offset.unwrap_or(0).to_string()),
                ("limit", 50.to_string()), // 50 is the maximum
            ],
        )
        .unwrap();
        let resp = Client::new()
            .get(url)
            .header("cookie", &self.cookie)
            .header("Accept", "application/json")
            .send()
            .await
            .unwrap();
        let text_resp = resp.text().await.expect("Failed to get response text");
        let json_response = match serde_json::from_str::<ApiResponse>(&text_resp) {
            Ok(parsed) => parsed,
            Err(err) => {
                println!("Failed to parse response {}", err);
                panic!("Failed to parse response: {}", text_resp);
            }
        };
        let items = json_response
            .data
            .iter()
            .map(|item| crate::Music {
                title: item.title.clone(),
                author: item.artist.name.clone(),
                thumbnail: Some(item.album.cover.clone()),
                url: Some(item.link.clone()),
                date: None,
                album: Some(item.album.title.clone()),
            })
            .collect();
        let next_offset = match json_response.next {
            Some(next) => {
                let url = url::Url::parse(&next).expect("Failed to parse next url");
                let query = url.query().unwrap();
                url::form_urlencoded::parse(query.as_bytes())
                    .find(|(key, _)| key == "index")
                    .map(|(_, value)| value.parse::<u64>().unwrap())
            }
            None => None,
        };
        log::info!("Next offset: {:?}", next_offset);
        (items, next_offset)
    }
}

impl crate::Platform for DeezerPlatform {
    async fn init() -> Self {
        let cookie = input(
            "Please enter your deezer cookie",
            "MUSIC_EXPORTER_DEEZER_COOKIE",
        )
        .expect("COOKIE is required");
        let user_id = input(
            "Please enter your deezer user id",
            "MUSIC_EXPORTER_DEEZER_USER_ID",
        )
        .expect("USER_ID is required");
        Self { cookie, user_id }
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
