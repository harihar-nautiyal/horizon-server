use serde::{Deserialize};
#[derive(Deserialize)]
pub struct AdminCommand {
    pub action: String,
    #[serde(flatten)]
    pub payload: serde_json::Value,
}