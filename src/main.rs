extern crate reqwest;

use std::io::Read;

const TWR_ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

fn main() {
    let html = download_url(TWR_ARCHIVE_URL);
    println!("{}", html);
}

fn download_url(url: &str) -> String {
    let mut response = reqwest::get(url).expect("Failed to send request");

    let mut buf = String::new();
    response.read_to_string(&mut buf).expect("Failed to read response");

    buf
}
