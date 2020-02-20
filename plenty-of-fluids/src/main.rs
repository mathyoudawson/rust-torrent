extern crate rustc_serialize;
extern crate bencode;

use rustc_serialize::{Encodable, Decodable};

use bencode::{encode, Decoder, Bencode};

use std::fs;

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct TorrentMetadata{
    info_hash:   std::collections::BTreeMap<String, TorrentMetadataInfo>,
    announce:    String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq)]
struct TorrentMetadataInfo {
    piece_hashes: [u8; 20],
    piece_length: i32,
    length:      i32,
    name:        String,
}

fn main() {
    const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";

    let bencoded_metadata = fs::read(TORRENT_PATH).unwrap();

    let bencode: bencode::Bencode = bencode::from_vec(bencoded_metadata).unwrap();


    println!("{}", bencode);

    let mut decoder = Decoder::new(&bencode);
    // parse_single_or_multi_file_metadata(&bencode);
    let result: TorrentMetadata = Decodable::decode(&mut decoder).unwrap();
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

