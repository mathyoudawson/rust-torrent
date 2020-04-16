use std::time::Duration;
use std::net::{SocketAddrV4, SocketAddr};
use std::io::prelude::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::mpsc;

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

pub struct PeerConnection {
    // stream: TcpStream,
    bitfield: Vec<u8>,
    outgoing_tx: mpsc::Sender<message::Message>,
}

pub struct PeerConnections {
    connections: Vec<PeerConnection>,
    peer_incoming_rxs: Vec<mpsc::Receiver<message::Message>>,
}

impl PeerConnection {
    pub fn new(peer: &tracker::Peer,
               metadata: &parser::TorrentMetadata) -> (PeerConnection, mpsc::Receiver<message::Message>) {
        let (incoming_tx, incoming_rx) = mpsc::channel(1024);
        let (outgoing_tx, outgoing_rx) = mpsc::channel(1024);

        let socket_addr = SocketAddr::from(SocketAddrV4::new(peer.ip, peer.port));
        tokio::spawn(peer_connection_background(socket_addr.clone(), metadata.clone(), incoming_tx, outgoing_rx));

        (PeerConnection {
            // stream,
            bitfield: Vec::new(),
            outgoing_tx,
        }, incoming_rx)
    }

}

async fn peer_connection_background(socket_addr: SocketAddr,
                                    metadata: parser::TorrentMetadata,
                                    mut incoming_tx: mpsc::Sender<message::Message>,
                                    mut outgoing_rx: mpsc::Receiver<message::Message>)
    -> Result<(), String> {
    let mut tcp_stream = perform_handshake(&socket_addr, &metadata).await.map_err(|e| e.to_string())?;

    loop {
        while let Some(outgoing_message) = outgoing_rx.try_recv().ok() {
            println!("[unimplemented] send_message {:?}", outgoing_message);
        }

        let message = receive_message(&mut tcp_stream).await?;
        println!("[info] received peer message: {:?}", message);

        incoming_tx.send(message).await.unwrap();
    }
}

struct ReceivedMessages {
    peer_rxs: Vec<mpsc::Receiver<message::Message>>,
}

impl PeerConnections {
    pub fn received_messages(&mut self) -> impl futures::Stream<Item=message::Message> {
        assert!(!self.peer_incoming_rxs.is_empty(), "you are already listening to received messages");

        ReceivedMessages { peer_rxs: std::mem::replace(&mut self.peer_incoming_rxs, Vec::new()) }
    }
}

impl futures::Stream for ReceivedMessages {
    type Item = message::Message;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, _context: &mut std::task::Context)
        -> std::task::Poll<Option<message::Message>> {
        for peer in self.peer_rxs.iter_mut() {
            println!("pollig peer");

            if let Some(message) = peer.try_recv().ok() {
                println!("    -> has message");
                return std::task::Poll::Ready(Some(message));
            } else {
                println!("    -> has message");
            }
        }

        std::task::Poll::Pending
    }
}

pub async fn connect_to_peers(peers: &Vec<tracker::Peer>, metadata: &parser::TorrentMetadata)
    -> Result<PeerConnections, std::io::Error> {
    // we should spawn a thread for each peer - look into rayon for this
    let mut connected_peers: Vec<PeerConnection> = Vec::new();
    let mut peer_rxs = Vec::new();

    for peer in peers {
        println!{"{:?}", peer};

        // currently this sends and receives handshake
        // perhaps these should be called independantly

        //First message should always be bitfield
        // let message = match receive_message(stream) {
        //     Ok(m) => m,
        //     Err(e) => { println!("Error: {:?}", e);
        //         continue; },
        // };

        // message::message_handler(message);

        let (connected_peer, incoming_rx) = PeerConnection::new(peer, metadata);

        connected_peers.push(connected_peer);
        peer_rxs.push(incoming_rx);
    }

    Ok(PeerConnections {
        connections: connected_peers,
        peer_incoming_rxs: peer_rxs,
    })
}

async fn perform_handshake(socket_addr: &SocketAddr, metadata: &parser::TorrentMetadata) -> Result<TcpStream, std::io::Error> {

    let mut tcp_stream: tokio::net::TcpStream = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(socket_addr)).await??;

    const DEFAULT_PSTR: &str = "BitTorrent protocol";

    let handshake = Handshake {
        pstr: DEFAULT_PSTR.to_string(),
        info_hash: metadata.info_hash.to_owned(),
        peer_id: "plenty-of-fluid00001".to_string(),
    };

    let handshake_bytes = handshake.to_bytes().clone();

    tcp_stream.write(handshake_bytes.as_slice()).await?;
    println!("Awaiting response");

    match receive_handshake(&mut tcp_stream, metadata.info_hash.to_owned()).await {
        Ok(()) => return Ok(tcp_stream),
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e)),
    }
}

async fn receive_handshake(stream: &mut TcpStream, our_info_hash: Vec<u8>) -> Result<(), String> {
    let pstrlen = read_n(stream, 1).await?;
    read_n(stream, pstrlen[0] as u32).await?; // ignore pstr
    read_n(stream, 8).await?; // ignore reserved
    let info_hash = read_n(stream, 20).await?;
    let _peer_id = read_n(stream, 20).await?;

    {
        // validate info hash
        if info_hash != our_info_hash
        {
            return Err("Invalid info hash".to_string());
        }
    }

    Ok(())
}

async fn receive_message(stream: &mut TcpStream) -> Result<message::Message, String> {
    let mut stream = stream;
    // first 4 bytes indicate the size of the message
    let message_size = bytes_to_u32(&read_n(&mut stream, 4).await?);

    if message_size > 0 {
      let message = &read_n(&mut stream, message_size).await?;
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

async fn read_n(stream: &mut TcpStream, bytes_to_read: u32) -> Result<Vec<u8>, String> {
    let buf = read_n_to_buf(stream, bytes_to_read).await?;
    Ok(buf)
}

async fn read_n_to_buf(stream: &mut TcpStream, bytes_to_read: u32) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; bytes_to_read as usize];

    if bytes_to_read == 0 {
        return Ok(Vec::new());
    }

    let bytes_read = stream.read_exact(&mut buf).await;
    match bytes_read {
        Ok(0) => Err("Socket Closed".to_string()),
        Ok(n) if n == bytes_to_read as usize => Ok(buf),
        Ok(n) => {
            assert_eq!(n, bytes_to_read as usize);
            unreachable!("partial read unexpected");
        },
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
