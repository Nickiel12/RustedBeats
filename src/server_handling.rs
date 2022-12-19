use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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
    pub fn has_path(self: &Self) -> bool {
        self.path.is_some()
    }

    pub fn has_title(self: &Self) -> bool {
        self.title.is_some()
    }

    pub fn has_artist(self: &Self) -> bool {
        self.artist.is_some()
    }

    pub fn has_album(self: &Self) -> bool {
        self.album.is_some()
    }

    pub fn has_album_artist(self: &Self) -> bool {
        self.album_artist.is_some()
    }

    pub fn is_empty(self: &Self) -> bool {
        return self.path.is_none()
            && self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.album_artist.is_none();
    }
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
    GetList(String),
    SwitchTo(PartialTag),
    GetStatus,
}

pub fn handle_request(socket_message: String) -> Result<UIRequest, serde_json::Error> {
    println!("Recieved a socket message: {}", socket_message);
    let request: UIRequest = serde_json::from_str(&socket_message)?;
    Ok(request)
}
