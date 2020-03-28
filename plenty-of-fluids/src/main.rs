extern crate bencode;

use bencode::{Bencode};
use bencode::util::ByteString;
use url::Url;
use std::collections::*;
use std::io::prelude::*;
use std::process::Command;
use sha1::{Sha1, Digest};

use std::fs;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct TorrentMetadata{
    info: TorrentMetadataInfo,
    info_hash: Vec<u8>,
    announce: String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct TorrentMetadataInfo {
    pieces: [u8; 20],
    piece_length: i64,
    length:      i64,
    name:        String,
}

fn main() {
    //const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";
    const TORRENT_PATH: &str = "test.torrent";

    let bencoded_metadata: Vec<u8> = fs::read(TORRENT_PATH).unwrap();

    let bencode: bencode::Bencode = bencode::from_vec(bencoded_metadata).unwrap();

    let metadata = parse_torrent_file(&bencode).unwrap();
    // println!("Parsed torrent file: {:?}", metadata);

    let tracker_query = match build_tracker_query(metadata){
        Ok(query) => query,
        Err(e) => panic!("Could not build tracker query: {}", e),
    };  //unwrap();

    match execute_tracker_query(tracker_query) {
        Ok(response) => println!("Successful query: {}", response),
        Err(e) => panic!("Request error: {}", e),
    }
}

fn parse_torrent_file(bencode: &bencode::Bencode) -> Result<TorrentMetadata, String> {
    println!("bencode is : {}", bencode);
    let mut top_level_dict = if let Bencode::Dict(dict) = bencode {
        dict.clone()
    } else {
        panic!("top  level bencode should be a dict");
    };

    let info_bytes: Vec<u8> = get_field_as_bencoded_bytes(&top_level_dict, "info")?;

    // compute sha
    let mut hasher = Sha1::new();
    hasher.input(info_bytes);

    let info_hash = hasher.result().to_vec();
    println!("new info hash: {:?}", info_hash);

     println!("tld: {}", bencode);
    let announce = top_level_dict.remove(&bencode::util::ByteString::from_str("announce")).unwrap().to_string();
    let info_hashish = top_level_dict.remove(&bencode::util::ByteString::from_str("info")).unwrap();

    let mut info_dict = if let Bencode::Dict(ref dict) = info_hashish { dict.clone() } else { panic!("darn") };

    let pieces = match info_dict.remove(&bencode::util::ByteString::from_str("pieces")).unwrap() {
        bencode::Bencode::ByteString(v) => { 
            let mut pieces_array = [0u8; 20];
            for (place, element) in pieces_array.iter_mut().zip(v.iter()) {
                *place = *element;
            }
            pieces_array

        },
        _ => panic!("Not a bytestring"),
    };

    let piece_length =  match info_dict.remove(&bencode::util::ByteString::from_str("piece length")).unwrap() {
        bencode::Bencode::Number(n) => n,
        _ => panic!("Not a number"),
    };

    let info_name = info_dict.remove(&bencode::util::ByteString::from_str("name")).unwrap().to_string();

    let info_length: i64 = match info_dict.remove(&bencode::util::ByteString::from_str("length")).unwrap() {
        Bencode::Number(n) => n,
        _ => panic!("invalid torrent file length"),
    };

    let mut metadata = TorrentMetadata {
        announce,
        info_hash,
        info: TorrentMetadataInfo  {
            length: info_length,
            name: info_name,
            piece_length: piece_length,
            pieces: pieces,
        },
    };

    metadata.announce = "http://localhost:4040/announce".to_owned();

    Ok(metadata)
}

fn get_field_as_bencoded_bytes(dict: &BTreeMap<bencode::util::ByteString, bencode::Bencode>, field: &str) -> Result<Vec<u8>, String> {
    let raw_field = match dict.get(&ByteString::from_str(field)) {
        Some(a) => a,
        None => return Err("Could not find field".to_string()),
    };

    Ok(raw_field.to_bytes().unwrap())
}

// fn parse_single_or_multi_file_metadata(bencode: &bencode::Bencode)
//     -> Result<String, String> {
//         match bencode {
//             Bencode::Dict(entries) => {
//                 panic!("entries: {:#?}", entries);
//             },
//
//             _ => panic!("corrupted"),
//         }
// }

fn dirty_ruby_urlencode_hack(bytes: &[u8])
    -> String {
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join("temp-plenty-of-fluids.bin");

        let mut temp_file = std::fs::File::create(&temp_file_path).unwrap();

        temp_file.write_all(bytes).unwrap();
        drop(temp_file);

        let urlencoded_process = Command::new("ruby")
            .arg("-rcgi")
            .arg("-e")
            .arg("puts CGI.escape(ARGF.read)")
            .arg(temp_file_path)
            .output()
            .expect("Failed to execute command");

        let stdout = urlencoded_process.stdout;
        let stdout_bytes: Result<Vec<u8>, _>  = stdout.bytes().collect();
        let stdout_bytes = stdout_bytes.unwrap();
        let stdout_str = String::from_utf8(stdout_bytes).expect("Ruby did not produce valid UTF-8");

        stdout_str
}


fn build_tracker_query(torrent: TorrentMetadata) ->  Result<url::Url, reqwest::Error> { // reqwest::Error> {
    let formatted_url = if torrent.announce.starts_with("s") {
        let mut url: String = torrent.announce.chars().skip(2).collect();
        url.truncate(url.len() - 1);
        url
    }
    else {
        torrent.announce.clone()
    };

    let info_hash = dirty_ruby_urlencode_hack(&torrent.info_hash);

    let query = Url::parse_with_params(&formatted_url,
        &[
        ("info_hash", &info_hash[..]),
        ("peer_id", "plenty-of-fluid00001"),
        ("port", "6881"),
        ("uploaded", "0"),
        ("downloaded", "0"),
        ("compact", "0"),
        ("left", &torrent.info.length.to_string()),
        ]).unwrap();


    println!("URL: {:?}", query);

    // let resp = reqwest::blocking::get(&query.to_string())?
    //     .json::<HashMap<String, String>>()?;

    Ok(query)
}

fn execute_tracker_query(query: url::Url) -> Result<String, reqwest::Error> {
    // Currently the url encoding logic is encoding '%' as '%25'. Need to look into this
    let sanitized_query = &str::replace(&str::replace(&query.to_string(), "%25", "%"), "%0A", "");
    //let sanitized_query = &str::replace(&query.to_string(), "%25", "%");
    println!("query is : {}", sanitized_query);
    let response = reqwest::blocking::get(sanitized_query)?
        .text();

    Ok(response.unwrap())
}


fn bytes_to_hash_str(data: &[u8]) -> String {
    // println!("data: {:?}", data);
    let pieces: Vec<String> = data.iter().map(|byte| {
        format!("%{:02x}", byte)
    }).collect();

    // println!("pieces: {:?}", pieces);
    pieces.join("")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn url_encoding()  {
        assert_eq!(bytes_to_hash_str(b"hello"),
        "%68%65%6c%6c%6f");

    }
}

