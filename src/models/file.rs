use bson::DateTime;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub id: String,
    pub filename: String,
    pub download_url: String,
    pub uploaded_at: DateTime,
}