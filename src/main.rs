use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use reqwest;
use sha1::{Digest, Sha1};
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::net::TcpStream;
use rand::Rng;
use tokio;
use serde_bencode;

use crate::bencoding::decode_bencoded_value;

mod models;
mod bencoding;

fn parse_torrent_file(path: &Path) -> Result<models::TorrentFile, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let torrent: models::TorrentFile = serde_bencode::de::from_bytes(&bytes)?;
    Ok(torrent)
}

fn info_hash_generator(content: &models::TorrentFile) -> String {
    let info_encoded = serde_bencode::to_bytes(&content.info).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&info_encoded);
    let info_hash = hasher.finalize();
    hex::encode(info_hash)
}

fn info_hash_url_encoded(content: &models::TorrentFile) -> String {
    let info_encoded = serde_bencode::to_bytes(&content.info).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&info_encoded);
    let info_hash = hasher.finalize();
    percent_encode(&info_hash, NON_ALPHANUMERIC).to_string()
}

// Usage: your_program.sh decode "<encoded_value>"
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let file_path = Path::new(&args[2]);
        let content: models::TorrentFile = parse_torrent_file(file_path).expect("Could not parse file");
        println!(
            "Tracker URL: {}\nLength: {}",
            content.announce, content.info.length
        );

        let info_hash = info_hash_generator(&content);
        println!("Info Hash: {}", info_hash);
        println!("Piece Length: {}", content.info.piece_length);
        println!("Piece Hashes:\n");
        for hash in content.info.pieces.chunks(20) {
            println!("{}", hex::encode(&hash));
        }
    } else if command == "peers" {
        let file_path = Path::new(&args[2]);
        let content: models::TorrentFile = parse_torrent_file(file_path).expect("Could not parse file");
        let info_hash = info_hash_url_encoded(&content);

        let tracker = models::TrackerRequest {
            peer_id: String::from("00112233445566778899"),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: content.info.length,
            compact: 1
        };
        println!("Info Hash:{}", info_hash);
        let params = serde_urlencoded::to_string(tracker).unwrap();
        let tracker_url = format!("{}?{}&info_hash={}", content.announce, params, info_hash);
        println!("{}", tracker_url);
        let bytes = reqwest::get(tracker_url).await.unwrap().bytes().await.unwrap();
        let response: models::TrackerResponse = serde_bencode::de::from_bytes(bytes.as_ref()).expect("msg");

        for chunk in response.peers.chunks_exact(6) {
            let ip_bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            let ip_address = std::net::Ipv4Addr::from(ip_bytes);

            let port_bytes: [u8; 2] = [chunk[4], chunk[5]];
            let port = u16::from_be_bytes(port_bytes);
            println!("{}:{}", ip_address, port);
        }
    } else if command=="handshake"{ 
        let file_path = Path::new(&args[2]);
        let content: models::TorrentFile = parse_torrent_file(file_path).expect("Could not parse file");
        
        let addr= &args[3];
        let info_hash = {
            let mut hasher = Sha1::new();
            hasher.update(serde_bencode::to_bytes(&content.info).unwrap());
            hasher.finalize().to_vec()
        };
        assert_eq!(info_hash.len(), 20);

        // Generate random peer ID (20 bytes)
        let mut peer_id = [0u8; 20];
        rand::rng().fill(&mut peer_id);

        // Connect to peer
        let mut stream = TcpStream::connect(&addr).expect("Couldn't connect to the server...");
        println!("Connected to peer {}", addr);

        // Build handshake message
        let mut handshake = Vec::new();
        handshake.push(19u8); // length of protocol string
        handshake.extend_from_slice(b"BitTorrent protocol");
        handshake.extend_from_slice(&[0u8; 8]); // reserved bytes
        handshake.extend_from_slice(&info_hash); // info hash (20 bytes)
        handshake.extend_from_slice(&peer_id); // peer id (20 bytes)

        // Send handshake
        stream.write(&handshake).expect("Couldn't Write to stream...");
        println!("Handshake sent");

        // Read handshake back (68 bytes total)
        let mut response = [0u8; 68];
        stream.read_exact(&mut response).expect("Couldn't read response...");

        // Extract peer id from response
        let received_peer_id = &response[48..68];
        println!("Peer ID: {}", hex::encode(received_peer_id));
    } else {
        println!("unknown command: {}", args[1]);
    }
}
