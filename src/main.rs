use rodio::{Decoder, OutputStream, Sink};
use server_handling::UIRequest;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::PathBuf;
use std::thread::spawn;
use std::{fs::File, net::TcpListener};
use tungstenite::accept;
use tungstenite::protocol::WebSocket;

use clap::Parser;
use dirs_next;

use crate::db_operations::{DatabaseRequest, PartialTag};

pub mod db_operations;
pub mod file_operations;
pub mod server_handling;

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

    let db_path: PathBuf = ["/", "home", "nixolas", "RustedBeats.db"].iter().collect();

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

    let test_tag = PartialTag {
        title: Some("%bees%".to_string()),
        ..PartialTag::default()
    };

    let test_file = dbo
        .get(&DatabaseRequest {
            search_type: db_operations::SearchType::Like,
            search_tag: test_tag,
        })
        .unwrap()
        .unwrap();

    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // open an audio sink
    let sink = Sink::try_new(&stream_handle).unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open(test_file[0].path.clone()).unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    // Play the sound directly on the device
    sink.pause();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    /*
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    println!("is binary?: {:?}", msg.is_binary());
                    println!("msg: {:?}", msg);
                }
            }
        });
    }
    */

    let mut sockets = Vec::<WebSocket<TcpStream>>::new();
    loop {
        if let Ok((stream, addr)) = server.accept() {
            println!("New socket connected from: {}", addr);
            //TODO: handle this error
            sockets.push(accept(stream).unwrap());
            println!("len = {}", sockets.len());
        }

        if sockets.len() == 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("sleeping");
        }

        for i in 0..sockets.len() {
            if let Ok(mess) = sockets[i].read_message() {
                println!("got a message from a socket");
                if mess.is_text() {
                    println!("It was a text message!");
                    match server_handling::handle_request(mess.into_text().unwrap()) {
                        Err(error) => {
                            println!("There was an error decoding the message: {:?}", error)
                        }
                        Ok(req) => match req {
                            UIRequest::Play => sink.play(),
                            UIRequest::Pause => sink.pause(),
                            UIRequest::Skip(skip_direction) => todo!(),
                            UIRequest::GetList => todo!(),
                            UIRequest::SwitchTo(partia_tag) => todo!(),
                            UIRequest::GetStatus => todo!(),
                        },
                    }
                }
            }
        }
    }
}
