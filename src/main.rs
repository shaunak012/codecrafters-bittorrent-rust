use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::env;
use std::path::Path;
use sha1::{Digest, Sha1};
use reqwest;
use tokio;

// Available if you need it!
// use serde_bencode

#[derive(Serialize, Deserialize)]
struct TorrentInfo{
    length: u64,
    name: String,
    #[serde(rename="piece length")]
    piece_length: u64,
    #[serde(with="serde_bytes")]
    pieces: Vec<u8>
}

#[derive(Serialize, Deserialize)]
struct TorrentFile{
    announce:String,
    info: TorrentInfo
}

#[derive(Serialize, Deserialize)]
struct TrackerResponse{
    interval: i32,
    peers: String
}

fn bencode_ending_index(encoded_value: &str) -> usize {
    if encoded_value.chars().next().unwrap().is_digit(10) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<usize>().unwrap();
        return colon_index + 1 + number ;
    } else if encoded_value.starts_with("i"){
        let ending_index = encoded_value.find('e').expect("Invalid bencoded integer format");
        return ending_index+1 ;
    } else if encoded_value.starts_with("l") || encoded_value.starts_with("d"){
        let mut counter = 0;
        let mut i = 0;
        let chars: Vec<char> = encoded_value.chars().collect();
        while i < chars.len(){
            match chars[i] {
                'l' => counter+= 1,
                'd' => counter+= 1,
                'i' => {
                    // println!("Entry at {}",i);
                    i+=1;
                    while chars[i].is_digit(10){
                        i+=1;
                    }
                    // println!("Exit at {}",i);
                },
                'e' => {
                    // println!("Entry at {}, {}",i, &counter);
                    counter-=1;
                    if counter ==0{
                        break;
                    }
                    // println!("Exit at {}, {}",i, &counter);
                },
                _ =>{
                    // println!("Entry at {}",i);
                    if chars[i].is_digit(10) {
                        let mut j = i;
                        while chars[j] != ':' { j += 1; }
                        let len: usize = encoded_value[i..j].parse().unwrap();
                        i = j + len; 
                    }
                    // println!("Exit at {}",i);
                }
            }
            i+=1;
        }
        return i+1;
    } else{
        panic!("Invalid string : {}",encoded_value);
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> Value {
    // If encoded_value starts with a digit, it's a number
    
    let ending_index = bencode_ending_index(encoded_value);
    // println!("Ending Index: {}",ending_index);
    if encoded_value.chars().next().unwrap().is_digit(10) {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.find(':').unwrap();
        let string = &encoded_value[colon_index + 1..ending_index];
        return Value::String(string.to_string());
    } else if encoded_value.starts_with("i"){
        let number_part = &encoded_value[1..ending_index-1];
        let number=number_part.parse::<i64>().unwrap();
        return Value::Number(number.into());
    } else if encoded_value.starts_with("l"){
        let mut list = vec![];
        let mut current_index = 1; 
        while current_index < ending_index-1{
            // println!("List left: {}",&encoded_value[current_index..]);
            let element_end= bencode_ending_index(&encoded_value[current_index..]);
            // println!("Element End: {}",current_index+element_end);
            list.push(decode_bencoded_value(&encoded_value[current_index..]));
            current_index+=element_end;
        }
        return Value::Array(list);
    } else if encoded_value.starts_with("d"){
        let mut list = serde_json::Map::new();
        let mut current_index =1;
        while current_index < ending_index-1{
            let key_end= bencode_ending_index(&encoded_value[current_index..]);
            let key = match decode_bencoded_value(&encoded_value[current_index..]){
                    Value::String(k) => k,
                    k => {
                        panic!("dict keys must be strings, not {k:?}");
                    }
                };
            current_index+=key_end;
            let value_end = bencode_ending_index(&encoded_value[current_index..]);
            let value = decode_bencoded_value(&encoded_value[current_index..]);
            current_index+=value_end;
            list.insert(key,value);
        }
        return Value::Object(list)
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

fn parse_torrent_file(path:&Path)-> Result<TorrentFile,Box<dyn std::error::Error>>{
    let bytes = std::fs::read(path)?;
    let torrent: TorrentFile = serde_bencode::de::from_bytes(&bytes)?;
    Ok(torrent)
}

fn info_hash_generator(content: &TorrentFile) -> Digest{
    let info_encoded = serde_bencode::to_bytes(&content.info).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&info_encoded);
    let info_hash = hasher.finalize();
    return info_hash;
}

// Usage: your_program.sh decode "<encoded_value>"
#[tokio::main]
fn main() {
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
        let content: TorrentFile = parse_torrent_file(file_path).expect("Could not parse file");
        println!("Tracker URL: {}\nLength: {}", content.announce, content.info.length);

        let info_hash=info_hash_generator(&content);
        println!("Info Hash: {}", hex::encode(info_hash)); 
        println!("Piece Length: {}",content.info.piece_length);
        println!("Piece Hashes:\n");
        for hash in content.info.pieces.chunks(20) {
            println!("{}", hex::encode(&hash));
        }
    } else if command == "peers" {
        let client = reqwest::Client::new();
        
        let file_path = Path::new(&args[2]);
        let content: TorrentFile = parse_torrent_file(file_path).expect("Could not parse file");
        let info_hash=info_hash_generator(&content);

        let params = serde_urlencoded::to_string({"info_hash": urlencoding::encode(info_hash), "peer_id":String::from("00112233445566778899"), "port":6881, "uploaded":0, "downloaded":0, "left":content.info.length, "compact":1});
        let response = reqwest::get(content.announce).query(&params).send().await?;
        let body: TrackerResponse = response.text().await?;

        for chunk in body.peers.as_bytes().chunks_exact(6) {
            let ip_bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            let ip_address = std::net::Ipv4Addr::from(ip_bytes);

            let port_bytes: [u8; 2] = [chunk[4], chunk[5]];
            let port = u16::from_be_bytes(port_bytes);
            println!("{}:{}", ip_address, port);
        }
    } else {
        println!("unknown command: {}", args[1]);
    }
}
