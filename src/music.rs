#[derive(Debug, Ord, Eq, PartialOrd, serde::Deserialize, serde::Serialize, Clone)]
pub struct Music {
    pub author: String,
    pub title: String,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
    pub date: Option<String>,
    pub album: Option<String>,
}

impl Music {
    fn normalized_title(&self) -> String {
        self.title.trim().to_lowercase()
    }

    fn normalized_author(&self) -> String {
        self.author.trim().to_lowercase()
    }
}

impl PartialEq for Music {
    fn eq(&self, other: &Self) -> bool {
        self.normalized_title() == other.normalized_title()
            && self.normalized_author() == other.normalized_author()
    }
}

use std::collections::HashSet;

pub fn unique_music(music_vec: Vec<Music>) -> Vec<Music> {
    let mut unique_vec = Vec::new();
    let mut seen = HashSet::new();

    for music in music_vec {
        if seen.insert((
            music.normalized_title().clone(),
            music.normalized_author().clone(),
        )) {
            unique_vec.push(music);
        } else {
            log::info!("Duplicate: {} by {}", music.title, music.author);
        }
    }

    unique_vec
}
