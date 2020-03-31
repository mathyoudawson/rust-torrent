extern crate bencode;

use url::Url;
use std::io::prelude::*;
use std::process::Command;
mod hash;
mod decoder;
mod parser;

use std::fs;

fn main() {
    //const TORRENT_PATH: &str = "src/archlinux-2020.02.01-x86_64.iso.torrent";
    const TORRENT_PATH: &str = "test.torrent";

    let bencoded_metadata: Vec<u8> = fs::read(TORRENT_PATH).unwrap();

    let metadata = parser::parse_bencoded_torrent(bencoded_metadata).unwrap();

    let tracker_query = match build_tracker_query(metadata){
        Ok(query) => query,
        Err(e) => panic!("Could not build tracker query: {}", e),
    };  //unwrap();

    match execute_tracker_query(tracker_query) {
        Ok(response) => println!("Successful query: {}", response),
        Err(e) => panic!("Request error: {}", e),
    }
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

        stdout_str
}


fn build_tracker_query(torrent: parser::TorrentMetadata) ->  Result<url::Url, reqwest::Error> { // reqwest::Error> {
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
