use crate::message_types::UIRequest;
use log::{debug, error, info, trace, warn, LevelFilter};

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
