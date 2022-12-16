use std::path::PathBuf;

use clap::Parser;
use dirs_next;

pub mod db_operations;
pub mod file_operations;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Set the root of your music library (defaults to user music dir)
    #[arg(short, long)]
    root_directory: Option<String>,

    /// Specify a specific configuration file
    #[arg(short, long)]
    configuration_file: Option<String>,

    /// Specify a specific database file
    #[arg(short, long)]
    database_file: Option<String>,

    /// Start the server without a front end
    #[arg(long)]
    no_webserver: bool,

    /// Run the database in memory alone
    #[arg(long)]
    no_save: bool,

    /// Delete an existing database file (wherever it looks on startup)
    #[arg(long)]
    reset_database: bool,
}

fn main() {
    let cli = Cli::parse();

    // settings = confy settings
    let music_dir: String;
    if cli.root_directory.is_some() {
        music_dir = cli.root_directory.clone().unwrap();
    } else {
        music_dir = String::from(dirs_next::audio_dir().unwrap().to_str().unwrap());
    }

    let music_scanner = file_operations::MusicScanner::new(music_dir);

    let db_path: PathBuf = ["home", "nixolas", "RustedBeats.db"].iter().collect();

    let dbo = db_operations::DBObject::new(&db_path, false).unwrap();

    for file_batch in music_scanner {
        for filepath in file_batch {
            if filepath.to_string_lossy().ends_with(".wav") {
                continue;
            } else {
                let tag = file_operations::get_tag(&filepath).unwrap();
                dbo.save_tag(&tag).unwrap();
            }
        }
    }

    println!("{:?}", cli);
}
