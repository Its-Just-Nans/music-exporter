//! Deezer platform implementation
//! Useful link https://developers.deezer.com/api

use reqwest::Client;
use std::{future::Future, pin::Pin};

use super::types::ApiResponse;
use crate::{custom_env, errors::MusicExporterError, utils::input_env, Music};

/// Deezer platform implementation
#[derive(Default)]
pub struct DeezerPlatform {
    /// Deezer cookie
    cookie: String,

    /// Deezer user id
    user_id: String,
}

impl DeezerPlatform {
    /// Get playlist items
    /// # Errors
    /// Error if the response is not a valid json
    async fn get_playlist_items(
        &self,
        offset: Option<u64>,
    ) -> Result<(Vec<crate::Music>, Option<u64>), MusicExporterError> {
        let url = url::Url::parse_with_params(
            &format!("https://api.deezer.com/user/{}/tracks", self.user_id),
            &[
                ("index", offset.unwrap_or(0).to_string()),
                ("limit", 50.to_string()), // 50 is the maximum
            ],
        )?;
        let resp = Client::new()
            .get(url)
            .header("cookie", &self.cookie)
            .header("Accept", "application/json")
            .send()
            .await?;
        let text_resp = resp.text().await?;
        let json_response = serde_json::from_str::<ApiResponse>(&text_resp)?;
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
                let url = url::Url::parse(&next)?;
                let query = url
                    .query()
                    .ok_or(MusicExporterError::new("Cannot get query"))?;
                url::form_urlencoded::parse(query.as_bytes())
                    .find(|(key, _)| key == "index")
                    .map(|(_, value)| value.parse::<u64>())
                    .transpose()?
            }
            None => None,
        };
        log::info!("Next offset: {:?}", next_offset);
        Ok((items, next_offset))
    }
}

impl crate::Platform for DeezerPlatform {
    fn try_new() -> Pin<Box<dyn Future<Output = Result<Self, MusicExporterError>> + Send>> {
        Box::pin(async {
            let cookie = input_env(
                "Please enter your deezer cookie",
                custom_env!("DEEZER_COOKIE"),
            )?;
            let user_id = input_env(
                "Please enter your deezer user id",
                custom_env!("DEEZER_USER_ID"),
            )?;

            Ok(Self { cookie, user_id })
        })
    }

    fn get_list<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Music>, MusicExporterError>> + Send + 'a>> {
        Box::pin(async {
            let mut items = Vec::new();
            let mut page_next = None;
            loop {
                let (new_items, next_page) = self.get_playlist_items(page_next).await?;
                items.extend(new_items);
                page_next = next_page;
                if page_next.is_none() {
                    break;
                }
            }
            Ok(items)
        })
    }
}
