use serde::{Serialize, Deserialize};
use bson::DateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    id: String,
    device: String,
    registered_at: DateTime,
    updated_at: DateTime,
    last_online: DateTime
}