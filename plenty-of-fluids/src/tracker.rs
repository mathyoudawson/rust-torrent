extern crate url;

use super::parser;
use url::form_urlencoded;
use std::borrow::Cow;

pub fn query_tracker(metadata: parser::TorrentMetadata) -> Result<String, reqwest::Error> {
    let query = build_tracker_query(metadata)?;

    execute_tracker_query(query)
}

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
