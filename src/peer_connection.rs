use std::time::Duration;
use std::net::{SocketAddrV4, SocketAddr, TcpStream};
use std::io::prelude::*;

use super::tracker;
use super::parser;
use super::message;

struct Handshake {
   pstr: String,
   info_hash: Vec<u8>, //pass info metadata to extract these fields - string may not be correct. Also pstr should maybe be initialized with a constant..?
   peer_id: String,
}

impl Handshake {
    fn to_bytes(&self) -> Vec<u8> {
       let mut bytes: Vec<u8> = Vec::new(); 
       bytes.push(self.pstr.len() as u8); // length of protocol id
       bytes.append(&mut self.pstr.as_bytes().to_vec()); // protocol id
       bytes.append(&mut vec![0u8; 8]); // 8 bytes used to indicate extensions we dont support yet
       bytes.append(&mut self.info_hash.clone()); // 8 bytes used to indicate extensions we dont support yet
       bytes.append(&mut self.peer_id.as_bytes().to_vec()); // 8 bytes used to indicate extensions we dont support yet
       bytes
    }
}

pub fn connect_to_peers(peers: &Vec<tracker::Peer>, metadata: &parser::TorrentMetadata) {
    // we should spawn a thread for each peer - look into rayon for this

    for peer in peers {
        println!{"{:?}", peer};

        // currently this sends and receives handshake
        // perhaps these should be called independantly
        let stream = match initiate_handshake(&peer, &metadata) {
            Ok(stream) => { 
                println!("TcpStream connected!");
                stream },
            Err(e) => { println!("Error: {}", e);
                continue; },
        };

        // First message should always be bitfield
        let bitfield = match receive_message(stream) {
            Ok(m) => m,
            Err(e) => { println!("Error: {:?}", e);
                continue; },
        };

        message::message_handler(bitfield);
    }
}

fn initiate_handshake(peer: &tracker::Peer, metadata: &parser::TorrentMetadata) -> Result<TcpStream, std::io::Error> {
    let socket = SocketAddr::from(SocketAddrV4::new(peer.ip, peer.port));

    let mut tcp_stream = match TcpStream::connect_timeout(&socket, Duration::from_secs(3)) {
        Ok(stream) => stream,
        Err(e) => {
            println!("Couldn't connect to the peer");
            return Err(e)
        },
    };

    const DEFAULT_PSTR: &str = "BitTorrent protocol";

    let handshake = Handshake {
        pstr: DEFAULT_PSTR.to_string(),
        info_hash: metadata.info_hash.to_owned(),
        peer_id: "plenty-of-fluid00001".to_string(),
    };

    let handshake_bytes = handshake.to_bytes().clone();

    tcp_stream.write(handshake_bytes.as_slice())?;
    println!("Awaiting response");

    match receive_handshake(&mut tcp_stream, metadata.info_hash.to_owned()) {
        Ok(()) => return Ok(tcp_stream),
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e)),
    }
}

fn receive_handshake(stream: &mut TcpStream, our_info_hash: Vec<u8>) -> Result<(), String> {
    let pstrlen = read_n(stream, 1)?;
    read_n(stream, pstrlen[0] as u32)?; // ignore pstr
    read_n(stream, 8)?; // ignore reserved
    let info_hash = read_n(stream, 20)?;
    let _peer_id = read_n(stream, 20)?;

    {
        // validate info hash
        if info_hash != our_info_hash
        {
            return Err("Invalid info hash".to_string());
        }
    }

    Ok(())
}

fn receive_message(stream: TcpStream) -> Result<message::Message, String> {
    let mut stream = stream;
    // first 4 bytes indicate the size of the message
    let message_size = bytes_to_u32(&read_n(&mut stream, 4)?);

    if message_size > 0 {
      let message = &read_n(&mut stream, message_size)?;
      Ok(message::identify_message(message[0], &message[1..]))
    } else {
       Ok(message::Message::KeepAlive) // do nothing 
    }
}

// stole this logc - this feels weird
const BYTE_0: u32 = 256 * 256 * 256;
const BYTE_1: u32 = 256 * 256;
const BYTE_2: u32 = 256;
const BYTE_3: u32 = 1;

fn bytes_to_u32(bytes: &[u8]) -> u32 {
    bytes[0] as u32 * BYTE_0 +
    bytes[1] as u32 * BYTE_1 +
    bytes[2] as u32 * BYTE_2 +
    bytes[3] as u32 * BYTE_3
}

fn read_n(stream: &mut TcpStream, bytes_to_read: u32) -> Result<Vec<u8>, String> {
    let mut buf = vec![];
    read_n_to_buf(stream, &mut buf, bytes_to_read)?;
    Ok(buf)
}

fn read_n_to_buf(stream: &mut TcpStream, buf: &mut Vec<u8>, bytes_to_read: u32) -> Result<(), String> {
    if bytes_to_read == 0 {
        return Ok(());
    }

    let bytes_read = stream.take(bytes_to_read as u64).read_to_end(buf);
    match bytes_read {
        Ok(0) => Err("Socket Closed".to_string()),
        Ok(n) if n == bytes_to_read as usize => Ok(()),
        Ok(n) => read_n_to_buf(stream, buf, bytes_to_read - n as u32),
        Err(e) => Err(e.to_string())
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    //
    // #[test]
    // fn correctly_build_bindable_ip()  {
    //     assert_eq!(build_bindable_ip("192.168.1.1", &8080u16), "192.168.1.1:8080")
    // }
}
