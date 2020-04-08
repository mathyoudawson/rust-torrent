extern crate bencode;
use bencode::{Bencode};
use std::collections::*;
use bencode::util::ByteString;

pub fn get_number_from_bencode(dict: &BTreeMap<bencode::util::ByteString, bencode::Bencode>, field: &str) -> i64 {
    let bencode = get_field(dict, field).unwrap();

    match bencode {
        Bencode::Number(n) => n,
        _ => panic!("Expected Number!"),
    }
}

pub fn get_string_from_bencode(dict: &BTreeMap<bencode::util::ByteString, bencode::Bencode>, field: &str) -> String {
    let bencode = get_field(dict, field).unwrap();

    bencode.to_string()
}

pub fn get_field(dict: &BTreeMap<bencode::util::ByteString, bencode::Bencode>, field: &str) -> Result<bencode::Bencode, String> {
    match dict.get(&ByteString::from_str(field)) {
        Some(a) => Ok(a.to_owned()),
        None => return Err("Could not find field".to_string()),
    }
}

pub fn get_field_as_bencoded_bytes(dict: &BTreeMap<bencode::util::ByteString, bencode::Bencode>, field: &str) -> Result<Vec<u8>, String> {
    let raw_field = match dict.get(&ByteString::from_str(field)) {
        Some(a) => a,
        None => return Err("Could not find field".to_string()),
    };

    Ok(raw_field.to_bytes().unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_correct_number_from_bencode_dict()  {
        let num = 13i64;
        let mut dict = BTreeMap::new();
        dict.insert(ByteString::from_str("test"), bencode::Bencode::Number(num));

        assert_eq!(get_number_from_bencode(&dict, "test"), num)
    }

    #[test]
    fn get_correct_string_from_bencode_dict() {
        let string = "test string";
        let string_bytes = string.as_bytes().to_vec();
        let mut dict = BTreeMap::new();
        dict.insert(ByteString::from_str("test"), bencode::Bencode::ByteString(string_bytes));

        assert_eq!(get_string_from_bencode(&dict, "test"), "s\"test string\"")
    }

    #[test]
    fn get_correct_field(){
        let num = bencode::Bencode::Number(13i64);
        let mut dict = BTreeMap::new();
        dict.insert(ByteString::from_str("test"), num.clone());

        assert_eq!(get_field(&dict, "test"), Result::Ok(num))
    }

    #[test]
    fn get_correct_field_as_bencoded_bytes(){
        let mut dict = BTreeMap::new();
        let bytes = bencode::Bencode::ByteString(vec![0u8; 20]);
        dict.insert(ByteString::from_str("test"), bytes);

        assert_eq!(get_field_as_bencoded_bytes(&dict, "test").is_ok(), true)
    }
}
