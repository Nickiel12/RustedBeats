use id3::{Tag, TagLike};
use scan_dir::ScanDir;
use std::{path::{PathBuf, Path}, ffi::OsStr};
use log::warn;

use crate::message_types::ItemTag;

const SUPPORTED_FILETYPES: [&str; 1] = ["mp3"];

/// The object that iteratively and recursively scans the directories
///
/// The way to instantiate this struct is to call new with the root directory.
///
/// # Examples
/// ```rust
/// let music_scanner = MusicScanner::new("/home/urs/Music/")
/// ```
pub struct MusicScanner {
    dirs: Vec<PathBuf>,
}

impl MusicScanner {
    pub fn new(root: String) -> Self {
        MusicScanner {
            dirs: vec![root.into()],
        }
    }
}

impl Iterator for MusicScanner {
    type Item = Vec<PathBuf>;

    /// This function pops the top most of MusicScanner.dirs internal Vec, and scans a directory
    /// It returns a Vector of found file paths while updating the internal directory list
    ///
    /// # Examples
    /// ```rust
    /// let music_scanner = MusicScanner::new("/home/exampleusr/Music");
    /// let mut file = music_scanner.next();
    /// println!(file);
    /// >>> "/home/usr/Music/file_1.mp3"
    ///
    /// file = music_scanner.next();
    /// println!(file);
    /// >>> "/home/usr/Music/file_2.mp3"
    /// ```
    ///
    fn next(&mut self) -> Option<Self::Item> {
        let mut files = vec![];

        let target = match self.dirs.pop() {
            Some(val) => val,
            None => {
                return None;
            }
        };

        // scan the currect dir for other directories for later scanning
        ScanDir::dirs()
            .read(target.clone(), |iter| {
                for (entry, _name) in iter {
                    self.dirs.push(entry.path());
                }
            })
            .unwrap();

        // scan the current dir for normal files
        ScanDir::files()
            .read(target.clone(), |iter| {
                for (entry, _name) in iter {
                    match entry.path().to_str() {
                        Some(path) => {
                            // Check if the file has a valid extension
                            match Path::new(path).extension().and_then(OsStr::to_str) {
                                Some(extension) => {
                                    if SUPPORTED_FILETYPES.contains(&extension){
                                        files.push(entry.path());
                                    }
                                },
                                // Does not have a valid extension
                                None => continue
                            }
                        },
                        None => {
                            warn!("Error encountered in directory: {:?}", target);
                            warn!("There was an error getting the path to {:?}", path);
                        }
                    }
                }
            })
            .unwrap();

        // return the found files
        Some(files)
    }
}

/// Returns the music information from a filepath
pub fn get_tag(filepath: &PathBuf) -> Result<ItemTag, id3::Error> {
    let tag = Tag::read_from_path(filepath)?;

    let mut output_tag = ItemTag {
        ..ItemTag::default()
    };
    output_tag.path = filepath.to_string_lossy().into_owned();

    // Get a bunch of frames...
    if let Some(artist) = tag.artist() {
        output_tag.artist = artist.to_string();
    }
    if let Some(title) = tag.title() {
        output_tag.title = title.to_string();
    }
    if let Some(album) = tag.album() {
        output_tag.album = album.to_string();
    }

    Ok(output_tag)
}
