use std::path::PathBuf;

use scan_dir::ScanDir;

const SUPPORTED_FILETYPES: [&str; 1] = ["mp3"];

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
