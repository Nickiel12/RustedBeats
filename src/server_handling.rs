use crate::message_types::{UIRequest, ItemTag, ServerResponse, PartialTag};
use log::info;
use tungstenite::protocol::WebSocket;
use std::net::TcpStream;

/// Pass a
pub fn handle_request(socket_message: String) -> Result<UIRequest, serde_json::Error> {
    info!("Recieved a socket message: {}", socket_message);
    let request: UIRequest = serde_json::from_str(&socket_message)?;

    Ok(request)
}

fn sanitize_input(input: UIRequest) -> Result<UIRequest, ()> {
    // if UIRequest is a search string, make sure it is
    // not empty
    // has no %, this is a fuzzy search, This program handles that, maybe replace * with %
    // has a type of request (e.g. "title search: value")
    Ok(UIRequest::Pause)
}

fn sanitize_tag(input: ItemTag) -> ItemTag {
    let mut output = ItemTag{
        ..ItemTag::default()
    };
    output.path = input.path.replace("'", "''");
    output.title = input.title.replace("'", "''");
    output.album = input.album.replace("'", "''");
    output.artist = input.artist.replace("'", "''");
    output.album_artist = input.album_artist.replace("'", "''");
    return output;
}

pub fn sanitize_partialtag(input: PartialTag) -> PartialTag {
    let mut output = PartialTag{
        ..PartialTag::default()
    };

    if input.path.is_some() {output.path = Some(input.path.unwrap().replace("'", "''"));};
    if input.title.is_some() {output.title = Some(input.title.unwrap().replace("'", "''"));};
    if input.album.is_some() {output.album = Some(input.album.unwrap().replace("'", "''"));};
    if input.artist.is_some() {output.artist = Some(input.artist.unwrap().replace("'", "''"));};
    if input.album_artist.is_some() {output.album_artist = Some(input.album_artist.unwrap().replace("'", "''"));};
    println!("output tag {:?}", output);
    return output;
}

pub fn write_to_socket(
    socket: &mut WebSocket<TcpStream>,
    message: String,
    results: Vec<ItemTag>,
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
