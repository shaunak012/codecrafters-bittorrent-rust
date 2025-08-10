use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub length: u64,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    pub announce: String,
    pub info: TorrentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerResponse {
    pub interval: u64,
    #[serde(with = "serde_bytes")]
    pub peers: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRequest {
    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: u64,
    pub compact: u8,
}