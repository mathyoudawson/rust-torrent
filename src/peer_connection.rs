use std::net::{TcpStream};
use std::time::Duration;
use std::net::{SocketAddrV4, SocketAddr};
use std::io::prelude::*;

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

pub fn initiate_handshake(peer: &tracker::Peer, metadata: &parser::TorrentMetadata) -> std::io::Result<()> {
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

    tcp_stream.write(handshake.to_bytes().as_slice())?;
    println!("Awaiting response");
    let response = tcp_stream.read(&mut vec![0; 128])?;
    println!("Response is: {:?}", response);

    Ok(())
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
