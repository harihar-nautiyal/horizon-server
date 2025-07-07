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

impl Admin {
    pub fn new(guid: String) -> Admin {
        let now = bson::DateTime::now();
        Admin {
            id: ObjectId::new(),
            guid,
            registered_at: now.clone(),
            updated_at: now.clone(),
            last_online: now.clone()
        }
    }
}