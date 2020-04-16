use super::*;

use futures::StreamExt;

pub async fn download_to_file(metadata: &parser::TorrentMetadata) {
    let peers = match tracker::get_peers(&metadata) {
        Ok(peers) => peers,
        Err(e) => panic!(e),
    };

    let mut connected_peers = peer_connection::connect_to_peers(&peers, &metadata).await.unwrap();

    // TODO:

    // connected_peers.interested(metadata);
    connected_peers.received_messages().for_each(|message| {
        println!("received message: {:?}", message);
        futures::future::ready(())
    }).await;
}

