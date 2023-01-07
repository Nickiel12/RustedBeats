use log::{debug, error, info, trace, warn, LevelFilter};
use simplelog::*;
use std::fs::File;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use tungstenite::accept;
use tungstenite::protocol::WebSocket;

use clap::Parser;
use dirs_next;

use crate::db_operations::DatabaseRequest;
pub mod db_operations;
pub mod file_operations;
pub mod message_types;
pub mod music_player;
pub mod server_handling;

use crate::db_operations::DBObject;
use crate::message_types::{PartialTag, ServerResponse, UIRequest};
use crate::music_player::MusicPlayer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Set the root of your music library (defaults to user music dir)
    #[arg(short, long)]
    root_directory: Option<String>,

    /// Specify a specific configuration file
    #[arg(short, long)]
    configuration_file: Option<String>,

    #[arg(long, default_value = "SousaLog.txt")]
    log_file: String,

    /// Specify a specific database file
    #[arg(short, long)]
    database_file: Option<String>,

    /// Run the database in memory alone
    #[arg(long, default_value = "false")]
    no_save: bool,

    /// Delete an existing database file (wherever it looks on startup)
    /// TODO: actually make this a thing
    #[arg(long)]
    reset_database: bool,
}

fn main() {
    let cli = Cli::parse();
    // settings = confy settings

    let log_file: String;

    log_file = "testing_log.txt".to_string();

    init_logger(log_file);

    let music_dir: String;
    if cli.root_directory.is_some() {
        music_dir = cli.root_directory.clone().unwrap();
    } else {
        music_dir = String::from(dirs_next::audio_dir().unwrap().to_str().unwrap());
    }

    let music_scanner = file_operations::MusicScanner::new(music_dir.clone());

    let db_path: PathBuf = ["/", "home", "nixolas", "RustedBeats.db"].iter().collect();

    info!("Opening database in memory mode: {}", cli.no_save);
    info!("Database file path is: {}", &db_path.to_string_lossy());
    let dbo = db_operations::DBObject::new(&db_path, cli.no_save).unwrap();

    info!("Starting file scan with root set to: {}", music_dir);
    for file_batch in music_scanner {
        for filepath in file_batch {
            debug!("checking file: {}", filepath.to_string_lossy());
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

    info!("Creating music player");
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let mut music_player = MusicPlayer::new(test_file[0].clone(), &stream_handle);

    info!("Opening Tcp Listener");
    let tcp_listener = TcpListener::bind("127.0.0.1:9001").unwrap();
    tcp_listener.set_nonblocking(true).unwrap();

    let mut sockets = Vec::<WebSocket<TcpStream>>::new();
    info!(
        "Socket listening on: {}",
        tcp_listener.local_addr().unwrap()
    );
    println!("Listening on {}", tcp_listener.local_addr().unwrap());

    loop {
        if let Ok((stream, addr)) = tcp_listener.accept() {
            stream.set_nonblocking(true).unwrap();

            info!("New socket connected from: {}", addr);
            println!("New socket connected from: {}", addr);

            match accept(stream) {
                Ok(sck) => sockets.push(sck),
                Err(_) => continue,
            }
        }

        if sockets.len() == 0 {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        // Need to get an asynchronous socket reader like tokio
        for i in 0..sockets.len() {
            match sockets[i].read_message() {
                Ok(mess) => {
                    if mess.is_text() {
                        match server_handling::handle_request(mess.into_text().unwrap()) {
                            Err(error) => {
                                println!("There was an error decoding the message: {:?}", error)
                            }
                            Ok(req) => handle_uirequest(
                                req,
                                &mut sockets[i],
                                &mut music_player,
                                &dbo,
                                &stream_handle,
                            )
                            .unwrap(),
                        }
                    }
                }
                Err(error) => match error {
                    tungstenite::Error::ConnectionClosed => {
                        println!("dropping socket");
                        let tmp = sockets.remove(i);
                        drop(tmp);
                    }
                    tungstenite::Error::Io(_) => {
                        if error.to_string().ends_with("(os error 32)") {
                            sockets.remove(i);
                        } else if error.to_string().ends_with("(os error 11)") {
                            continue;
                        } else if error
                            .to_string()
                            .ends_with("Trying to work with closed connection")
                        {
                            sockets.remove(i);
                        } else {
                            println!("There was an IO error: {}", error.to_string());
                        }

                        //panic!();
                        //continue;
                    }
                    _ => {
                        println!("A socket errored: {}", error.to_string());
                        sockets.remove(i);
                    }
                },
            }
        }
    }
}

fn handle_uirequest(
    request: UIRequest,
    socket: &mut WebSocket<TcpStream>,
    music_player: &mut MusicPlayer,
    dbo: &DBObject,
    stream_handle: &rodio::OutputStreamHandle,
) -> Result<(), String> {
    match request {
        UIRequest::Play => {
            music_player.play();
            write_to_socket(socket, "Player Resumed".to_string(), vec![]).unwrap();
        }
        UIRequest::Pause => {
            music_player.pause();
            write_to_socket(socket, "Player Paused".to_string(), vec![]).unwrap();
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
                None => socket.write_message("None".into()).unwrap(),
                Some(items) => {
                    write_to_socket(socket, "Here are the results:".to_string(), items).unwrap();
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
                    write_to_socket(socket, "No song found with that field!".to_string(), vec![])
                        .unwrap();
                }
                Some(items) => {
                    if items.len() > 1 {
                        write_to_socket(
                            socket,
                            "Multiple results found\nPlease be more specific".to_string(),
                            items,
                        )
                        .unwrap();
                    } else {
                        println!(
                            "Switching song to: '{}'",
                            items.get(0).unwrap().title.clone()
                        );

                        music_player
                            .change_now_playing(items.get(0).unwrap().clone())
                            .unwrap();
                        println!("{}", items.get(0).unwrap().path.clone());

                        write_to_socket(socket, "Switching now playing".to_string(), items)
                            .unwrap();

                        music_player.play();
                    }
                }
            }
        }
        UIRequest::GetStatus => todo!(),
    }

    Ok(())
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

pub fn init_logger(output_file: String) {
    // TODO: configure the log levels to something appropriate
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .unwrap();
}
