use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String, 
    pub agent: String,
    pub exp: i64,   
    pub iat: i64,
}

impl Claims {
    pub fn new(id: String, agent: String ,duration: Duration) -> Self {
        let iat = Utc::now().timestamp();
        let exp = (Utc::now() + duration).timestamp();
        Claims { id, agent, exp, iat }
    }
}