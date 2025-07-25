use actix_web::{web, HttpResponse};
use serde::{Serialize, Deserialize};
use bson::{DateTime};
use bson::oid::ObjectId;
use mongodb::Collection;
use bson::doc;
use chrono::Duration;
use futures_util::TryStreamExt;
use crate::models::app_state::AppState;
use crate::models::jwt::{Access, Claims};

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

    pub async fn get_all(collection: &Collection<Client>) -> mongodb::error::Result<Option<Vec<Client>>> {
        let cursor = collection.find(doc! {}).await?;
        let results: Vec<Client> = cursor.try_collect().await?;
        Ok(Some(results))
    }

    pub async fn get(id: &ObjectId, collection: &Collection<Client>) -> mongodb::error::Result<Option<Self>> {
        collection.find_one(doc! {"_id": id}).await
    } 

    pub async fn get_from_guid(guid: &String, collection: &Collection<Client>) -> mongodb::error::Result<Option<Self>> {
        collection.find_one(doc! { "guid": guid }).await
    }

    pub async fn insert(self, collection: &Collection<Client>) -> mongodb::error::Result<Self> {
        collection.insert_one(&self).await?;
        Ok(self)
    }

    pub async fn jwt_request(self, state: web::Data<AppState>) -> HttpResponse {
        match Claims::generate_jwt(self.id, self.guid, Access::Client, &state.jwt_secret, Duration::days(16)) {
            Ok(token) => HttpResponse::Ok().json(doc! { "token": token }),
            Err(e) => HttpResponse::InternalServerError().json(doc! {
                        "error": format!("JWT generation failed: {}", e)
        }),
        }
    }
}