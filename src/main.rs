extern crate bencode;

mod hash;
mod decoder;
mod parser;
mod tracker;
mod peer_connection;
mod message;

/// Bencoded torrent file data fixtures.
///
/// These are embedded as constants rather than loaded from the file system so
/// that the final executable can be distributed on its own, without the source code.
mod torrent_files {
    #![allow(dead_code)] // these are currently hardcoded.

    pub const ARCH_LINUX: &'static [u8] = include_bytes!("../res/torrent-file/archlinux-2020.02.01-x86_64.iso.torrent");
    pub const TEST: &'static [u8] = include_bytes!("../res/torrent-file/test.torrent");
}

fn main() {
    // TEST TORRENTS
    // let torrent_file = self::torrent_files::TEST;
    let torrent_file = self::torrent_files::ARCH_LINUX;

    let bencoded_metadata: Vec<u8> = torrent_file.to_owned();

    let metadata = parser::parse_bencoded_torrent(bencoded_metadata).unwrap();

    let peers = match tracker::get_peers(&metadata) {
        Ok(peers) => peers,
        Err(e) => panic!(e),
    };

    peer_connection::connect_to_peers(&peers, &metadata);
}
