use bencode::{Bencode};
use super::decoder;
use super::hash;


#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct TorrentMetadata{
    pub info: TorrentMetadataInfo,
    pub info_hash: Vec<u8>,
    pub announce: String,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct TorrentMetadataInfo {
    pub pieces: [u8; 20],
    pub piece_length: i64,
    pub length:      i64,
    pub name:        String,
}

pub fn parse_bencoded_torrent(bencoded_metadata: Vec<u8>) -> Result<TorrentMetadata, String> {
    let bencode: bencode::Bencode = bencode::from_vec(bencoded_metadata).unwrap();

    let top_level_dict = if let Bencode::Dict(dict) = bencode {
        dict.clone()
    } else {
        panic!("top  level bencode should be a dict");
    };

    let info_dict = top_level_dict.get(&bencode::util::ByteString::from_str("info")).unwrap();
    let info_dict = if let Bencode::Dict(ref dict) = info_dict { dict.clone() } else { panic!("Could not find info dict") };

    let pieces = match info_dict.get(&bencode::util::ByteString::from_str("pieces")).unwrap() {
        bencode::Bencode::ByteString(v) => { 
            let mut pieces_array = [0u8; 20];
            for (place, element) in pieces_array.iter_mut().zip(v.iter()) {
                *place = *element;
            }
            pieces_array

        },
        _ => panic!("Not a bytestring"),
    };

    let info_bytes = decoder::get_field_as_bencoded_bytes(&top_level_dict, "info")?;

    let metadata = TorrentMetadata {
        announce: decoder::get_string_from_bencode(&top_level_dict, "announce"),
        info_hash: hash::compute_sha1_hash(info_bytes),
        info: TorrentMetadataInfo  {
            length: decoder::get_number_from_bencode(&info_dict, "length"),
            name: decoder::get_string_from_bencode(&info_dict, "name"),
            piece_length: decoder::get_number_from_bencode(&info_dict, "piece length"),
            pieces: pieces,
        },
    };


    Ok(metadata)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn correctly_parse_torrent_file()  {
        let bencoded_metadata: Vec<u8> = crate::torrent_files::TEST.to_owned();
        assert_eq!(parse_bencoded_torrent(bencoded_metadata).is_ok(), true)
    }
}
