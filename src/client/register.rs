use actix_web::{post, Responder, web, HttpResponse};
use serde::{Deserialize};
use bson::{doc};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::models::app_state::AppState;
use crate::models::jwt::{Claims, Access};
use crate::models::client::Client;

#[derive(Deserialize)]
struct RegisterRequest {
    guid: String,
    agent: String,
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, data: web::Json<RegisterRequest>) -> impl Responder {
    let filter = doc! { "guid": &data.guid };
    match state.clients.find_one(filter.clone()).await {
        Ok(Some(client)) => {
            let claims = Claims::new(client.id.to_hex(), data.agent.clone(),  Access::Client, Duration::hours(1));
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

    let new_client = match Client::new(data.guid.clone(), data.agent.clone())
        .insert(&state.clients)
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
            "error": format!("Database error: {}", e)
        });
        }
    };

    match new_client.generate_jwt(&state.jwt_secret, Duration::hours(1)) {
        Ok(token) => HttpResponse::Ok().json(doc! { "token": token }),
        Err(e) => HttpResponse::InternalServerError().json(doc! {
            "error": format!("Failed to generate token: {}", e)
        }),
    }
}
