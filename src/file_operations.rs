use std::path::PathBuf;

use scan_dir::ScanDir;

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
    /// let music_scanner = MusicScanner::new("/home/usr/Music");
    /// let files = music_scanner.next();
    /// println!(files);
    /// >>> "/home/usr/Music/file_1.mp3"
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
        // TODO: Need to add filters once list of supported files is created
        ScanDir::files()
            .read(target, |iter| {
                for (entry, _name) in iter {
                    files.push(entry.path());
                }
            })
            .unwrap();

        // return the found files
        if files.len() > 0 {
            Some(files)
        } else {
            None
        }
    }
}

/// Recursivly scans the given root directory for files
/// WIP Function! All it does is print the directories
///
/// # Examples
///
/// ```rust
/// scan_music_dir("/home/Documents/RustedBeats");
///
///>>> "I found a directory src at .../src"
///>>> "I found a directory debug at .../debug"
///>>> "I found a file Cargo.toml at .../Cargo.toml"
///...
/// ```
///
pub fn scan_music_dir(root: String) -> Result<(), ()> {
    let mut directories = Vec::<PathBuf>::new();
    directories.push(root.into());

    while directories.len() != 0 {
        let target = match directories.pop() {
            Some(val) => val,
            None => {
                panic!("Whoa man this ai't right");
            }
        };

        ScanDir::dirs()
            .read(target.clone(), |iter| {
                for (entry, name) in iter {
                    directories.push(entry.path());
                    println!("I found a director {:?} at path {:?}", name, entry.path());
                }
            })
            .unwrap();

        ScanDir::files()
            .read(target, |iter| {
                for (entry, name) in iter {
                    println!("found file {:?} at path {:?}", name, entry.path());
                }
            })
            .unwrap();
    }

    Ok(())
}
