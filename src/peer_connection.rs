use std::net::{TcpStream};
use std::time::Duration;
use std::net::{SocketAddrV4, SocketAddr};
use std::io::prelude::*;
use std::str::from_utf8;

use super::tracker;
use super::parser;

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

pub fn initiate_handshake(peer: &tracker::Peer, metadata: &parser::TorrentMetadata) -> Result<TcpStream, std::io::Error> {
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

    // using 16 byte buffer, it works. dont know why other sizes don't...
    let mut data = [0u8; 16];

    // We can't always assume that we recieve a complete reponse
    // Currently we are just validating that the response matches our request.
    // In the future we should try append reponses until we have a full valid reponse.
    let mut successful_handshake = false;

    match tcp_stream.read(&mut data) {
        Ok(_) => {
            if eq(&data, handshake_bytes.as_slice()) {
                println!("Reply is ok!");
                successful_handshake = true;
            } else {
                let text = from_utf8(&data).unwrap();
                println!("Unexpected reply: {}", text);
            }
        },
        Err(e) => {
            println!("Failed to receive data: {}", e);
        }
    }

    if successful_handshake {
        Ok(tcp_stream)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Could not complete handshake with peer."))
    }
}

fn eq(arr: &[u8], other_arr: &[u8]) -> bool {
    arr.iter().zip(other_arr.iter()).all(|(a,b)| a == b) 
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
