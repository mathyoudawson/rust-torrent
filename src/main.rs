extern crate bencode;

mod hash;
mod decoder;
mod parser;
mod tracker;
mod peer_connection;

use std::fs;

fn main() {
    const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";

    // TEST TORRENTS
    //const TORRENT_PATH: &str = "test.torrent";
    //const TORRENT_PATH: &str = "ubuntu-18.04.4-desktop-amd64.iso.torrent";

    let bencoded_metadata: Vec<u8> = fs::read(TORRENT_PATH).unwrap();

    let metadata = parser::parse_bencoded_torrent(bencoded_metadata).unwrap();

    let _peers = match tracker::get_peers(metadata) {
        Ok(peers) => {
            for peer in &peers {
                println!{"{:?}", peer};

                match peer_connection::initiate_handshake(&peer) {
                    Ok(()) => println!("Successfull handshake"),
                    Err(_e) => println!("Some Err"),
                }
            }
            peers
        },
        Err(e) => panic!(e),
    };
}