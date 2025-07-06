use actix_web::{post, web, HttpResponse, Responder};
use bson::{doc, DateTime};
use bson::oid::ObjectId;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use crate::models::app_state::AppState;
use crate::models::client::Client;
use crate::models::jwt::{Access, Claims};

#[derive(Deserialize)]
struct RegisterRequest {
    guid: String,
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, data: web::Json<RegisterRequest>) -> impl Responder {
    let filter = doc! { "guid": &data.guid };
    match state.admins.find_one(filter.clone()).await {
        Ok(Some(client)) => {
            let claims = Claims::new(client.id.to_hex(), "Its admin bro".to_string(),  Access::Client, Duration::hours(1));
            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
            ) {
                Ok(token) => token,
                Err(e) => return HttpResponse::InternalServerError().json(doc! { "error": format!("Failed to generate token: {}", e) }),
            };
            return HttpResponse::Ok().json(doc! { "token": token });
        }
        Ok(None) => {}
        Err(e) => return HttpResponse::InternalServerError().json(doc! { "error": format!("Database error: {}", e) }),
    }

    let new_client = Client {
        id: ObjectId::new(),
        guid: data.guid.clone(),
        agent: "Its admin bro".to_string(),
        registered_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        updated_at: DateTime::from_millis(Utc::now().timestamp_millis()),
        last_online: DateTime::from_millis(Utc::now().timestamp_millis()),
    };

    match state.clients.insert_one(&new_client).await {
        Ok(_) => {
            let claims = Claims::new(new_client.id.to_hex(), "Its admin bro".to_string(), Access::Client,Duration::hours(1));
            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
            ) {
                Ok(token) => token,
                Err(e) => return HttpResponse::InternalServerError().json(doc! { "error": format!("Failed to generate token: {}", e) }),
            };
            HttpResponse::Ok().json(doc! { "token": token })
        }
        Err(e) => HttpResponse::InternalServerError().json(doc! { "error": format!("Database error: {}", e) }),
    }
}
