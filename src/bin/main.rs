use std::fs;
use std::time::Instant;
use torrent_client::bencode::Bencode;

fn main() {
    let file_content = fs::read("torrent_files/inception.torrent").unwrap_or_else(|err| {
        panic!("Error reading file: {:?}", err);
    });
    let start = Instant::now();
    for i in 0..10_000_000 {
        let content = Bencode::parse(&file_content);
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}
