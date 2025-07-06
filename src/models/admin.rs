use bson::DateTime;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct Admin {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub guid: String,
    pub registered_at: DateTime,
    pub updated_at: DateTime,
    pub last_online: DateTime
}