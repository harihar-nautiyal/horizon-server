use serde::{Serialize, Deserialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    pub status: Status,
    pub created_at: u32,
    pub updated_at: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Pending,
    Uploaded,
    Cancelled,
    Error
}