use serde::Serialize;

#[derive(Hash, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct OutputRecord {
    pub primary_artist: String,
    pub title: String,
    pub id: usize,
}
