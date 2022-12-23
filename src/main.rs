use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::net::TcpStream;
use std::path::PathBuf;
use std::{fs::File, net::TcpListener};
use tungstenite::accept;
use tungstenite::protocol::WebSocket;

use clap::Parser;
use dirs_next;

use crate::db_operations::DatabaseRequest;
pub mod db_operations;
pub mod file_operations;
pub mod message_types;
pub mod server_handling;

use crate::message_types::{PartialTag, ServerResponse, UIRequest};

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
        title: Some("bees".to_string()),
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
    let mut sink = Sink::try_new(&stream_handle).unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open(test_file[0].path.clone()).unwrap());
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    // Play the sound directly on the device
    sink.pause();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    let mut sockets = Vec::<WebSocket<TcpStream>>::new();
    println!("Listening on {}", server.local_addr().unwrap());
    loop {
        if let Ok((stream, addr)) = server.accept() {
            println!("New socket connected from: {}", addr);
            //TODO: handle this error
            sockets.push(accept(stream).unwrap());
        }

        if sockets.len() == 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        for i in 0..sockets.len() {
            if let Ok(mess) = sockets[i].read_message() {
                if mess.is_text() {
                    match server_handling::handle_request(mess.into_text().unwrap()) {
                        Err(error) => {
                            println!("There was an error decoding the message: {:?}", error)
                        }
                        Ok(req) => match req {
                            UIRequest::Play => {
                                sink.play();
                                write_to_socket(
                                    &mut sockets[i],
                                    "Player Paused".to_string(),
                                    vec![],
                                )
                                .unwrap();
                            }
                            UIRequest::Pause => {
                                sink.pause();
                                write_to_socket(
                                    &mut sockets[i],
                                    "Player Paused".to_string(),
                                    vec![],
                                )
                                .unwrap();
                            }
                            UIRequest::Skip(skip_direction) => todo!(),
                            UIRequest::Search(request) => {
                                println!("got a: {:?}", request);
                                let items = dbo
                                    .get(&DatabaseRequest {
                                        search_type: db_operations::SearchType::Like,
                                        search_tag: request,
                                    })
                                    .unwrap();

                                match items {
                                    None => sockets[i].write_message("None".into()).unwrap(),
                                    Some(items) => {
                                        write_to_socket(
                                            &mut sockets[i],
                                            "Here are the results:".to_string(),
                                            items,
                                        )
                                        .unwrap();
                                    }
                                }

                                //println!("got from db: {:?}", items);
                            }
                            UIRequest::SwitchTo(partial_tag) => {
                                let items = dbo
                                    .get(&DatabaseRequest {
                                        search_type: db_operations::SearchType::Like,
                                        search_tag: partial_tag,
                                    })
                                    .unwrap();

                                match items {
                                    None => {
                                        write_to_socket(
                                            &mut sockets[i],
                                            "No song found with that title!".to_string(),
                                            vec![],
                                        )
                                        .unwrap();
                                    }
                                    Some(items) => {
                                        if items.len() > 1 {
                                            write_to_socket(
                                                &mut sockets[i],
                                                "Please be more specific".to_string(),
                                                items,
                                            )
                                            .unwrap();
                                        } else {
                                            println!(
                                                "Switching song to: '{}'",
                                                items.get(0).unwrap().title.clone()
                                            );
                                            sink.stop();

                                            sink = Sink::try_new(&stream_handle).unwrap();
                                            let file = BufReader::new(
                                                File::open(items.get(0).unwrap().path.clone())
                                                    .unwrap(),
                                            );
                                            // Decode that sound file into a source
                                            let source = Decoder::new(file).unwrap();
                                            sink.append(source);
                                            println!("{}", items.get(0).unwrap().path.clone());

                                            write_to_socket(
                                                &mut sockets[i],
                                                "Switching now playing".to_string(),
                                                items,
                                            )
                                            .unwrap();

                                            sink.play();
                                            println!("{}", sink.is_paused());
                                        }
                                    }
                                }
                            }
                            UIRequest::GetStatus => todo!(),
                        },
                    }
                }
            }
        }
    }
}

fn write_to_socket(
    socket: &mut WebSocket<TcpStream>,
    message: String,
    results: Vec<message_types::ItemTag>,
) -> Result<(), tungstenite::Error> {
    socket.write_message(
        serde_json::to_string(&ServerResponse {
            message,
            search_results: results,
        })
        .unwrap()
        .into(),
    )
}
