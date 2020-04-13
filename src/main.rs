extern crate bencode;

mod hash;
mod decoder;
mod parser;
mod tracker;
mod peer_connection;

use std::fs;
use std::net::{TcpStream};

fn main() {
    const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";

    // TEST TORRENTS
    //const TORRENT_PATH: &str = "test.torrent";
    //const TORRENT_PATH: &str = "ubuntu-18.04.4-desktop-amd64.iso.torrent";

    let bencoded_metadata: Vec<u8> = fs::read(TORRENT_PATH).unwrap();

    let metadata = parser::parse_bencoded_torrent(bencoded_metadata).unwrap();

    let peers = match tracker::get_peers(&metadata) {
        Ok(peers) => peers,
        Err(e) => panic!(e),
    };

    // TODO: move this logic somewhere else.
    // Maybe in peer_connection itself.
    let mut tcp_streams: Vec<TcpStream> = Vec::new();

    for peer in &peers {
        println!{"{:?}", peer};

        match peer_connection::initiate_handshake(&peer, &metadata) {
            Ok(stream) => { 
                println!("TcpStream connected!");
                tcp_streams.push(stream); },
            Err(e) => println!("Error: {}", e),
        }
    }

    println!("{} peers ready for communication!", tcp_streams.len());
}
