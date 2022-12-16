use derive_more::From;
use rusqlite::{params, Connection, Params, Result};

use crate::file_operations::ItemTag;

/// Catch all Error for database creation errors
#[derive(From, Debug)]
pub enum DatabaseCreationError {
    RusqliteError(rusqlite::Error),
    IoError(std::io::Error),
}

pub struct DatabaseRequest {
    search_type: SearchType,
    search_tag: PartialTag,
}

pub enum SearchType {
    Where,
    Like,
}

pub struct PartialTag {
    path: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    album_artist: Option<String>,
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

/// The container object for the main database Connection
///
/// # Examples
/// ```rust
/// // Create a database only in memory
/// let db_object = DBObject::new("/there/is/no/file/saved", true);
///
/// let item = ItemTag {
///     path: "/path/to/music.mp3",
///     title: "An example song title",
///     artist: "An example artist",
///     album: "An example album",
///     album_artist: "An example album artist"
///  };
///
///  db_object.save_tag(&item).unwrap();
///
///  let request = DatabaseRequest {
///     search_type: SearchType::Where,
///     search_tag: PartialTag{
///         title: "An example song title".to_string()
///         ..default()
///     }
///  };
///
///  let ret = db_object.get(request)?;
///
///  assert!(ret.is_some());
///  assert_eq!(ret.unwrap().artist, "An example artist".to_string());
///   
///  ```
///
pub struct DBObject {
    pub conn: Connection,
}

impl DBObject {
    pub fn new(
        db_filepath: &std::path::PathBuf,
        in_memory: bool,
    ) -> Result<Self, DatabaseCreationError> {
        let conn: Connection;

        if in_memory {
            conn = Connection::open_in_memory()?;
        } else {
            std::fs::create_dir_all(db_filepath.parent().unwrap())?;
            conn = Connection::open(db_filepath)?;
        }

        // CREATE TABLE IF NOT EXISTS comment (
        //        id         INTEGER PRIMARY KEY,
        //        email      TEXT,
        //        author     TEXT,
        //        text       TEXT NOT NULL,
        //        timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP,
        //        content_id TEXT NOT NULL,
        //        parent     INTEGER
        //    )

        conn.execute(
            "CREATE TABLE IF NOT EXISTS musicinfo (
                path         TEXT PRIMARY KEY,
                title        TEXT NOT NULL,
                artist       TEXT,
                album        TEXT,
                album_artist TEXT

            )",
            params![],
        )?;

        Ok(DBObject { conn })
    }

    pub fn save_tag(self: &Self, tag: &ItemTag) -> Result<(), DatabaseCreationError> {
        self.conn.execute(
            "INSERT INTO musicinfo (path, title, artist, album, album_artist) VALUES ( ?1, ?2, ?3, ?4, ?5 )",
            params![tag.path, tag.title, tag.artist, tag.album, tag.album_artist],
        )?;
        Ok(())
    }

    /// Returns a vector of ItemTags that fulfil the requested query
    ///
    pub fn get(
        self: &Self,
        request: &DatabaseRequest,
    ) -> Result<Option<Vec<ItemTag>>, rusqlite::Error> {
        assert!(!request.search_tag.is_empty(), "There must be at least one field filled in the PartialItem. Use `get_all()` if you want the full table");

        let mut conditions = Vec::<String>::new();

        if request.search_tag.has_path() {
            conditions.push(format!(
                "path = '{}'",
                request.search_tag.path.clone().unwrap()
            ));
        }

        if request.search_tag.has_title() {
            conditions.push(format!(
                "title = '{}'",
                request.search_tag.title.clone().unwrap()
            ));
        }

        if request.search_tag.has_artist() {
            conditions.push(format!(
                "artist = '{}'",
                request.search_tag.artist.clone().unwrap()
            ));
        }

        if request.search_tag.has_album() {
            conditions.push(format!(
                "album = '{}'",
                request.search_tag.album.clone().unwrap()
            ));
        }

        if request.search_tag.has_album_artist() {
            conditions.push(format!(
                "album_artist = '{}'",
                request.search_tag.album_artist.clone().unwrap()
            ));
        }

        let condition: String;

        match request.search_type {
            SearchType::Where => {
                // Join
                condition = conditions.join(" AND ");
            }
            SearchType::Like => {
                condition = conditions.join(" AND ").replace("=", "LIKE");
            }
        }

        let req_string: String =
            String::from("SELECT * FROM musicinfo WHERE ") + condition.as_str();

        println!("Running sql: {}", req_string.clone());
        let mut stmt = self.conn.prepare(req_string.as_str())?;

        let ret_iter = stmt.query_map([], |row| {
            Ok(ItemTag {
                path: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                album_artist: row.get(4)?,
            })
        })?;

        let mut ret = Vec::<ItemTag>::new();
        for item in ret_iter {
            ret.push(item?);
        }

        if ret.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(ret))
        }
    }
}

#[test]
fn test_database_get_title() {
    let db_object =
        DBObject::new(&std::path::PathBuf::from("/there/is/no/file/saved"), true).unwrap();

    let item = ItemTag {
        path: "/path/to/music.mp3".to_string(),
        title: "An example song title".to_string(),
        artist: "An example artist".to_string(),
        album: "An example album".to_string(),
        album_artist: "An example album artist".to_string(),
    };

    db_object.save_tag(&item).unwrap();

    let request = DatabaseRequest {
        search_type: SearchType::Where,
        search_tag: PartialTag {
            title: Some("An example song title".to_string()),
            ..PartialTag::default()
        },
    };

    let ret = db_object.get(&request).unwrap();

    assert!(ret.is_some());
    assert_eq!(ret.unwrap()[0].artist, "An example artist".to_string());
}

#[test]
fn test_database_get_path() {
    let db_object =
        DBObject::new(&std::path::PathBuf::from("/there/is/no/file/saved"), true).unwrap();

    let item = ItemTag {
        path: "/path/to/music.mp3".to_string(),
        title: "An example song title".to_string(),
        artist: "An example artist".to_string(),
        album: "An example album".to_string(),
        album_artist: "An example album artist".to_string(),
    };

    db_object.save_tag(&item).unwrap();
    let request = DatabaseRequest {
        search_type: SearchType::Where,
        search_tag: PartialTag {
            path: Some("/path/to/music.mp3".to_string()),
            ..PartialTag::default()
        },
    };

    let ret = db_object.get(&request).unwrap();

    assert!(ret.is_some());
    assert_eq!(ret.unwrap()[0].artist, "An example artist".to_string());
}

#[test]
fn test_database_get_artist() {
    let db_object =
        DBObject::new(&std::path::PathBuf::from("/there/is/no/file/saved"), true).unwrap();

    let item = ItemTag {
        path: "/path/to/music.mp3".to_string(),
        title: "An example song title".to_string(),
        artist: "An example artist".to_string(),
        album: "An example album".to_string(),
        album_artist: "An example album artist".to_string(),
    };

    db_object.save_tag(&item).unwrap();
    let request = DatabaseRequest {
        search_type: SearchType::Where,
        search_tag: PartialTag {
            artist: Some("An example artist".to_string()),
            ..PartialTag::default()
        },
    };

    let ret = db_object.get(&request).unwrap();

    assert!(ret.is_some());
    assert_eq!(ret.unwrap()[0].artist, "An example artist".to_string());
}

#[test]
fn test_database_get_album() {
    let db_object =
        DBObject::new(&std::path::PathBuf::from("/there/is/no/file/saved"), true).unwrap();

    let item = ItemTag {
        path: "/path/to/music.mp3".to_string(),
        title: "An example song title".to_string(),
        artist: "An example artist".to_string(),
        album: "An example album".to_string(),
        album_artist: "An example album artist".to_string(),
    };

    db_object.save_tag(&item).unwrap();
    let request = DatabaseRequest {
        search_type: SearchType::Where,
        search_tag: PartialTag {
            album: Some("An example album".to_string()),
            ..PartialTag::default()
        },
    };

    let ret = db_object.get(&request).unwrap();

    assert!(ret.is_some());
    assert_eq!(ret.unwrap()[0].artist, "An example artist".to_string());
}

#[test]
fn test_database_get_album_artist() {
    let db_object =
        DBObject::new(&std::path::PathBuf::from("/there/is/no/file/saved"), true).unwrap();

    let item = ItemTag {
        path: "/path/to/music.mp3".to_string(),
        title: "An example song title".to_string(),
        artist: "An example artist".to_string(),
        album: "An example album".to_string(),
        album_artist: "An example album artist".to_string(),
    };

    db_object.save_tag(&item).unwrap();
    let request = DatabaseRequest {
        search_type: SearchType::Where,
        search_tag: PartialTag {
            album_artist: Some("An example album artist".to_string()),
            ..PartialTag::default()
        },
    };

    let ret = db_object.get(&request).unwrap();

    assert!(ret.is_some());
    assert_eq!(ret.unwrap()[0].artist, "An example artist".to_string());
}
