extern crate url;

use super::parser;
use url::form_urlencoded;
use std::borrow::Cow;
use bencode::{Bencode};
use curl::easy::Easy;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct Peer { // Peer should probably be in its own file as trackers and peers are different
    pub ip: Ipv4Addr, // url crate may give us another type here
    pub port: u16,
}

pub fn get_peers(metadata: &parser::TorrentMetadata) -> Result<Vec<Peer>, String> {
    let query = build_tracker_query(metadata)?;

    let response_bytes = execute_tracker_query(query).unwrap();

    let bencoded_response = bencode::from_vec(response_bytes).unwrap();

    let response_dict = if let Bencode::Dict(dict) = bencoded_response {
        dict.clone()
    } else {
        panic!("Reponse should be a dict!");
    };

    let peer_list = match response_dict.get(&bencode::util::ByteString::from_str("peers")).unwrap() {
        Bencode::ByteString(s) => s,
        _ => panic!("Not a ByteString"),
    };

    unmarshal_peers(peer_list)
}

fn unmarshal_peers(peers: &Vec<u8>) -> Result<Vec<Peer>, String> {
    const PEER_SIZE: u16 = 6;
    let mut unmarshalled_peers: Vec<Peer> = Vec::new();

    if peers.len() as u16 % PEER_SIZE != 0 {
        return Err("Received malformed peers".to_string());
    }

    let peer_chunks: Vec<&[u8]> = peers.chunks(PEER_SIZE as usize).collect();

    for chunk in peer_chunks {
        unmarshalled_peers.push(Peer{
            ip: Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]),
            port: u16::from_be_bytes(try_into_array(&chunk[4..])),
        });
    }

    Ok(unmarshalled_peers)
}

// This function is a sin. try_into will do this for me
fn try_into_array(slice: &[u8]) -> [u8; 2] {
    let mut array = [0u8; 2];
    for (&x, p) in slice.iter().zip(array.iter_mut()) {
        *p = x;
    }

    array
}

fn join_nums(nums: Vec<u8>, sep: &str) -> String {
   let nums_as_strings: Vec<String> = nums.iter().map(|n| n.to_string()).collect(); 

   nums_as_strings.join(sep)
}

fn build_tracker_query(metadata: &parser::TorrentMetadata) -> Result<String, String> {
    let formatted_url = if metadata.announce.starts_with("s") {
        let mut url: String = metadata.announce.chars().skip(2).collect();
        url.truncate(url.len() - 1);
        url
    }
    else {
        metadata.announce.clone()
    };

    let encoded_params = form_urlencoded::Serializer::new(String::new())
        .append_pair("peer_id", "plenty-of-fluid00001")
        .append_pair("port", "6881")
        .append_pair("uploaded", "0")
        .append_pair("downloaded", "0")
        .append_pair("compact", "0")
        .append_pair("left", &metadata.info.length.to_string())
        .encoding_override(Some(&|input| {
            if input != "!" {
                Cow::Borrowed(input.as_bytes())
            } else {
                Cow::Owned(metadata.info_hash.clone())
            }
        }))
    .append_pair("info_hash", "!")
        .finish();

    let query = [formatted_url.to_owned(), encoded_params].join("?");

    Ok(query)
}

fn execute_tracker_query(query: String) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&query).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    Ok(data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn correctly_unmarshal_peers()  {
        let bytes = vec![91, 8, 150, 67, 200, 213, 79, 68, 128, 152, 128, 137];
        assert_eq!(unmarshal_peers(&bytes).is_ok(), true);
     }

    // this naming is a hint to implement a from_bytes or something for peer
    #[test]
    fn unmarshals_bytes_into_peers() {
        let bytes = vec![91, 8, 150, 67, 200, 213, 79, 68, 128, 152, 128, 137];
        let peers = vec![Peer{ip: Ipv4Addr::new(91, 8, 150, 67), port: 51413},
                         Peer{ip: Ipv4Addr::new(79, 68, 128, 152), port: 32905}];

        let unmarshalled_peers = unmarshal_peers(&bytes).unwrap();
        assert_eq!(unmarshalled_peers[0].ip, peers[0].ip);
        assert_eq!(unmarshalled_peers[0].port, peers[0].port);
        assert_eq!(unmarshalled_peers[1].ip, peers[1].ip);
        assert_eq!(unmarshalled_peers[1].port, peers[1].port);
    }

    #[test]
    fn unmarshalling_returns_error_when_bytes_are_malformed(){
        let bytes = vec![91, 8, 150, 67, 200, 213, 79, 68, 128, 152, 128, 137, 123];
    
        assert_eq!(unmarshal_peers(&bytes).unwrap_err(), "Received malformed peers");
    }
}
