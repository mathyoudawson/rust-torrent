extern crate url;

use super::parser;
use url::form_urlencoded;
use std::borrow::Cow;
use bencode::{Bencode};
use bencode::util::ByteString;
use std::str;

pub struct Peer {
    ip: String, // url crate may give us another type here
    id: String,
    port: i64,
}

pub fn query_tracker(metadata: parser::TorrentMetadata) -> Result<String, reqwest::Error> {
    let query = build_tracker_query(metadata)?;

    let response = match execute_tracker_query(query) {
        Ok(s) => s,
        Err(e) => panic!("Tracker query failed: {}", e),
    };

    println!("here is response: {:?}", response);

    let bencoded_response = bencode::from_vec(response.as_bytes().to_vec()).unwrap(); 

    let response_dict = if let Bencode::Dict(dict) = bencoded_response {
        dict.clone()
    } else {
        panic!("top  level bencode should be a dict");
    };

    let peer_list = match response_dict.get(&bencode::util::ByteString::from_str("peers")).unwrap() {
        Bencode::List(l) => l,
        _ => panic!("Not a list"),
    };

    let mut decoded_peer_list: Vec<Peer> = Vec::new();

    for peer in peer_list {
        let peer_dict = if let Bencode::Dict(dict) = peer {
            dict.clone()
        } else {
            panic!("top  level bencode should be a dict");
        };

        let id_bytes: Vec<u8> = match peer_dict.get(&ByteString::from_str("peer id")).unwrap() {
            Bencode::ByteString(v) => v.to_owned(),
            _ => panic!("Peer id is not a bytestring"),
        };

        let id = match String::from_utf8(id_bytes) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let ip_bytes: Vec<u8> = match peer_dict.get(&ByteString::from_str("ip")).unwrap() {
            Bencode::ByteString(v) => v.to_owned(),
            _ => panic!("IP is not a bytestring"),
        };

        let ip = match String::from_utf8(ip_bytes) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let port: i64 = match peer_dict.get(&ByteString::from_str("port")).unwrap() {
            Bencode::Number(n) => n.to_owned(),
            _ => panic!("Port is not a number"),
        };

        println!("id: {:?}", id);
        println!("ip: {:?}", ip);
        println!("port: {:?}", port);

        decoded_peer_list.push(
            Peer {
                ip,
                id,
                port,
        }
        );
    }


    // let peer_list = unmarshal_peers(peers);

    // interval will dictate how often we are supposed to connect to the tracker to refresh the list of peers. Will be relevant when we open a port and communicate with peers
    //let interval = response_dict.get(&bencode::util::ByteString::from_str("interval")).unwrap();
    //println!("interval: {:?}", interval);

    Ok(response)
}

// fn unmarshal_peers(bencoded_peers: &Bencode) {
//     const peer_size: u8 = 6;
// }

fn build_tracker_query(metadata: parser::TorrentMetadata) -> Result<String, reqwest::Error> {
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

fn execute_tracker_query(query: String) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(&query)?
        .text();

    Ok(response.unwrap())
}

#[cfg(test)]
mod test {
    //use super::*;

    //#[test]
    // todo
    // fn get_correct()  {
    //     assert_eq!()
    // }
}
