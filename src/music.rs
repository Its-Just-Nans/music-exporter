//! Music struct and utility functions

/// Music struct
#[derive(Debug, Ord, Eq, PartialOrd, serde::Deserialize, serde::Serialize, Clone)]
pub struct Music {
    /// Author of the music
    pub author: String,

    /// Title of the music
    pub title: String,

    /// URL of the music
    pub url: Option<String>,

    /// Thumbnail of the music
    pub thumbnail: Option<String>,

    /// Duration of the music
    pub date: Option<String>,

    /// Album of the music
    pub album: Option<String>,
}

impl Music {
    /// normalized title
    fn normalized_title(&self) -> String {
        self.title.trim().to_lowercase()
    }

    /// normalized author
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

/// Remove duplicates from a vector of Music
pub fn unique_music(music_vec: Vec<Music>) -> Vec<Music> {
    let mut unique_vec = Vec::new();
    let mut seen = HashSet::new();
    let mut dup_count = 0;
    for music in music_vec {
        if seen.insert((
            music.normalized_title().clone(),
            music.normalized_author().clone(),
        )) {
            unique_vec.push(music);
        } else {
            log::debug!("Duplicate: {} by {}", music.title, music.author);
            dup_count += 1;
        }
    }
    log::info!("Duplicates: {}", dup_count);
    unique_vec
}
