use derive_more::From;
use rusqlite::{params, Connection, Params, Result};

use crate::file_operations::ItemTag;

#[derive(From, Debug)]
pub enum DatabaseCreationError {
    RusqliteError(rusqlite::Error),
    IoError(std::io::Error),
}

pub struct DBObject {
    pub conn: Connection,
}

impl DBObject {
    pub fn new(
        db_filepath: &std::path::PathBuf,
        in_memory: bool,
    ) -> Result<Self, DatabaseCreationError> {
        let conn: Connection;

        std::fs::create_dir_all(db_filepath.parent().unwrap())?;

        if in_memory {
            conn = Connection::open_in_memory()?;
        } else {
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
}
