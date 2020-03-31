extern crate bencode;

mod hash;
mod decoder;
mod parser;
mod tracker;

use std::fs;

fn main() {
    //const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";
    const TORRENT_PATH: &str = "test.torrent";

    let bencoded_metadata: Vec<u8> = fs::read(TORRENT_PATH).unwrap();

    let metadata = parser::parse_bencoded_torrent(bencoded_metadata).unwrap();

    match tracker::query_tracker(metadata) {
        Ok(response) => println!("Successful query: {}", response),
        Err(e) => panic!("Request error: {}", e),
    }
}
