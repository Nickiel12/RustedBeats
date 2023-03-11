use serde::{Deserialize, Serialize};

/// A struct that defines all the music tags supported by Sousa
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemTag {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
}

impl Default for ItemTag {
    fn default() -> Self {
        ItemTag {
            path: String::new(),
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            album_artist: String::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PartialTag {
    pub path: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
}

impl Default for PartialTag {
    fn default() -> Self {
        PartialTag {
            path: None,
            title: None,
            artist: None,
            album: None,
            album_artist: None,
        }
    }
}

impl PartialTag {
    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    pub fn has_title(&self) -> bool {
        self.title.is_some()
    }

    pub fn has_artist(&self) -> bool {
        self.artist.is_some()
    }

    pub fn has_album(&self) -> bool {
        self.album.is_some()
    }

    pub fn has_album_artist(&self) -> bool {
        self.album_artist.is_some()
    }

    pub fn is_empty(&self) -> bool {
        return self.path.is_none()
            && self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.album_artist.is_none();
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerResponse {
    pub message: String,
    pub search_results: Vec<ItemTag>,
}

#[derive(Serialize, Deserialize)]
pub enum SkipDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize)]
pub enum UIRequest {
    Play,
    Pause,
    Skip(SkipDirection),
    Search(PartialTag),
    SwitchTo(PartialTag),
    GetStatus,
}
