use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    Admin,
    Client,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub guid: String,
    pub access: Access,
    pub exp: i64,
    pub iat: i64,
}


impl Claims {
    pub fn new(id: String, guid: String, access: Access ,duration: Duration) -> Self {
        let iat = Utc::now().timestamp();
        let exp = (Utc::now() + duration).timestamp();
        Claims { id, guid, access, exp, iat }
    }
}