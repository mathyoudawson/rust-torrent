extern crate rustc_serialize;
extern crate bencode;

use rustc_serialize::{Encodable, Decodable};

use bencode::{encode, Decoder, Bencode};
use url::Url;
use std::collections::*;
use std::io::prelude::*;
use std::process::Command;
use sha1::{Sha1, Digest};

use std::fs;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct TorrentMetadata{
    info: TorrentMetadataInfo,
    announce: String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct TorrentMetadataInfo {
    piece_hashes: [u8; 20],
    piece_length: i32,
    length:      i64,
    name:        String,
}

fn main() {
    const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";

    let bencoded_metadata = fs::read(TORRENT_PATH).unwrap();

    let bencode: bencode::Bencode = bencode::from_vec(bencoded_metadata).unwrap();


    let result = parse_torrent_file(&bencode);
    println!("{:?}", result);

    //let mut decoder = Decoder::new(&bencode);
    // parse_single_or_multi_file_metadata(&bencode);
    //let result: TorrentMetadata = Decodable::decode(&mut decoder).unwrap();
}

fn parse_torrent_file(bencode: &bencode::Bencode)
    -> Result<TorrentMetadata, String> {
    let mut top_level_dict = if let Bencode::Dict(dict) = bencode {
        dict.clone()
    } else {
        panic!("top  level bencode should be a dict");
    };

    let announce = top_level_dict.remove(&bencode::util::ByteString::from_str("announce")).unwrap().to_string();
    println!("tld: {}", bencode);
    let info_hashish = top_level_dict.remove(&bencode::util::ByteString::from_str("info")).unwrap();
    let mut info_dict = if let Bencode::Dict(ref dict) = info_hashish { dict.clone() } else { panic!("darn") };

    for key in info_dict.keys().collect::<Vec<_>>() {
        println!("info: {}", key);

    }

    let info_name = info_dict.remove(&bencode::util::ByteString::from_str("name")).unwrap().to_string();
    let info_length: i64 = match info_dict.remove(&bencode::util::ByteString::from_str("length")).unwrap() {
        Bencode::Number(n) => n,
        _ => panic!("invalid torrent file length"),
    };

    println!("Info: {}", info_hashish);

    let mut metadata = TorrentMetadata {
        announce,
        info: TorrentMetadataInfo  {
            length: info_length,
            name: info_name,
            piece_length: 12,
            piece_hashes: [0; 20],
        },
    };
    println!("old tracker: {:?}", metadata.announce);
    metadata.announce = "http://localhost:4040/announce".to_owned();
    println!("new, patched in tracker: {:?}", metadata.announce);

    // println!("metadata:  {:#?}", metadata);

    for key in top_level_dict.keys() {
        println!("REMAINING KEY: {}", key);
    }

    match build_tracker_query(metadata){
        Ok(()) => println!("Result successful"),
        Err(r) => println!("Result unsuccessful {}", r),
    }  //unwrap();
    unimplemented!()
}

fn parse_single_or_multi_file_metadata(bencode: &bencode::Bencode)
    -> Result<String, String> {
    match bencode {
        Bencode::Dict(entries) => {
            panic!("entries: {:#?}", entries);
        },

        _ => panic!("corrupted"),
    }
}


fn dump(bencode: Bencode) -> Bencode {
    match &bencode {
        bencode::Bencode::ByteString(vec) => {
            // convert to actual string and print
        },
        _ => {
            println!("{:?}", bencode)
        },
    } 

    bencode
}

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
    //unimplemented!("urlencoded output: '{}'", stdout_str);
    // ruby -e 'puts CGI.escape(ARGF.read)'
    stdout_str
}


fn build_tracker_query(torrent: TorrentMetadata) -> Result<(), reqwest::Error> {
    let formatted_url = if torrent.announce.starts_with("s") {
        println!("AMENDING");
        let mut url: String = torrent.announce.chars().skip(2).collect();
        url.truncate(url.len() - 1);
        url
    }
    else {
        torrent.announce.clone()
    };

    let mut hasher = Sha1::new();
    hasher.input(encode(&torrent.info).unwrap());

    let hash = hasher.result();
    // let hash_str = make_url_encoded(&hash);
    let hash_str = dirty_ruby_urlencode_hack(&hash);

    //let info_hash: String = url::form_urlencoded::byte_serialize(&hash);
    println!("HASH STR: {:?}", hash_str);

    let query = Url::parse_with_params(&formatted_url,
        &[
            ("info_hash", &hash_str[..]),
            ("peer_id", "BLARGH1234"),
            ("port", "6881"),
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("compact", "0"),
            ("left", &torrent.info.length.to_string()),
        ]).unwrap();


    println!("URL: {:?}", query);

    // let resp = reqwest::blocking::get(&query.to_string())?
    //     .json::<HashMap<String, String>>()?;

    let resp = reqwest::blocking::get(&query.to_string())?
        .text();

    println!("Reponse: {:#?}", resp);
    Ok(())
}


fn make_url_encoded(data: &[u8]) -> String {
    let pieces: Vec<String> = data.iter().map(|byte| {
        format!("%{:02x}", byte)
    }).collect();

    pieces.join("")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn url_encoding()  {
        assert_eq!(make_url_encoded(b"hello"),
                   "%68%65%6c%6c%6f");

    }


}

