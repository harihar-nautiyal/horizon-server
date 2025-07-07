use bson::doc;
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    Admin,
    Client,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: ObjectId,
    pub guid: String,
    pub access: Access,
    pub exp: i64,
    pub iat: i64,
}


impl Claims {
    pub fn new(id: ObjectId, guid: String, access: Access ,duration: Duration) -> Self {
        let iat = Utc::now().timestamp();
        let exp = (Utc::now() + duration).timestamp();
        Claims { id, guid, access, exp, iat }
    }

    pub fn generate_jwt(id: ObjectId, guid: String, access: Access, secret: &str, duration: Duration) -> Result<String, JwtError> {
        let claims = Claims::new(
            id,
            guid.clone(),
            access,
            duration,
        );
        encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
    }

    pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(decoded.claims)
    }
}