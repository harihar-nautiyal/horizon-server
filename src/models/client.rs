use serde::{Serialize, Deserialize};
use bson::{DateTime, JavaScriptCodeWithScope};
use bson::oid::ObjectId;
use mongodb::Collection;
use crate::models::jwt::{Claims, Access};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use chrono::Duration;
use bson::doc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub guid: String,
    pub agent: String,
    pub registered_at: DateTime,
    pub updated_at: DateTime,
    pub last_online: Option<DateTime>
}

impl Client {
    pub fn new(guid: String, agent: String) -> Self {
        let now = DateTime::now();
        Client {
            id: ObjectId::new(),
            guid,
            agent,
            registered_at: now.clone(),
            updated_at: now.clone(),
            last_online: None
        }
    }

    pub async fn get(&self, collection: &Collection<Client>) -> mongodb::error::Result<Option<Self>> {
        collection.find_one(doc! { "_id": &self.id }).await
    }

    pub async fn insert(self, collection: &Collection<Client>) -> mongodb::error::Result<Self> {
        collection.insert_one(&self).await?;
        Ok(self)
    }

    pub fn generate_jwt(&self, secret: &str, duration: Duration) -> Result<String, JwtError> {
        let claims = Claims::new(
            self.id.to_hex(),
            self.guid.clone(),
            Access::Client,
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