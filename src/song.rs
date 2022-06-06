use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Song {
    pub id: usize,
    pub api_path: String,
    pub full_title: String,
    pub primary_artist: Value,
    pub title: String,
    pub url: String,
}
