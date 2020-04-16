use super::*;

pub fn download_to_file(metadata: &parser::TorrentMetadata) {
    let peers = match tracker::get_peers(&metadata) {
        Ok(peers) => peers,
        Err(e) => panic!(e),
    };

    let connected_peers = peer_connection::connect_to_peers(&peers, &metadata);
}

