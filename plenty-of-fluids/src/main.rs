extern crate rustc_serialize;
extern crate bencode;

use rustc_serialize::{Encodable, Decodable};

use bencode::{encode, Decoder, Bencode};

use std::fs;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
struct TorrentMetadata{
    info_hash: TorrentMetadataInfo,
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

    let metadata = TorrentMetadata {
        announce,
        info_hash: TorrentMetadataInfo  {
            length: info_length,
            name: info_name,
            piece_length: 12,
            piece_hashes: [0; 20],
        },
    };

    println!("metadata:  {:#?}", metadata);

    for key in top_level_dict.keys() {
        println!("REMAINING KEY: {}", key);
    }

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

