use crate::db_operations::PartialTag;

pub enum SkipDirection {
    Forward,
    Backward,
}

pub enum UIRequest {
    Play,
    Pause,
    Skip(SkipDirection),
    GetList,
    SwitchTo(PartialTag),
    GetStatus,
}

pub fn handle_request(socket_message: String) -> Result<UIRequest, ()> {
    println!("Recieved a socket message: {}", socket_message);
    Ok(UIRequest::Play)
}
