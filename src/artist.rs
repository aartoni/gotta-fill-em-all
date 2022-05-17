use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Artist {
    pub name: String,
    pub id: usize,
}
