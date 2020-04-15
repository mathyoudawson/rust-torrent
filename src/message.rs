struct MessagePayload {
    message_id: u8,
    message: Message,
    payload: Vec<u8>,
}

#[derive(Debug)]
pub enum Message {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    KeepAlive,
}

pub fn identify_message(message_id: u8) -> Message {
    match message_id {
        0 => Message::Choke,
        1 => Message::Unchoke,
        2 => Message::Interested,
        3 => Message::NotInterested,
        4 => Message::Have,
        5 => Message::Bitfield,
        6 => Message::Request,
        7 => Message::Piece,
        8 => Message::Cancel,
        _ => panic!("Uknown message"),
    }
}

// pub fn message_handler(msg: MessagePayload) {
// }
