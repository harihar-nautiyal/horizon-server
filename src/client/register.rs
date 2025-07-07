use actix_web::{post, web, HttpResponse, Responder};
use bson::{doc};
use serde::Deserialize;
use crate::models::app_state::AppState;
use crate::models::client::Client;

#[derive(Deserialize)]
struct RegisterRequest {
    guid: String,
    agent: String
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, data: web::Json<RegisterRequest>) -> impl Responder {
    match Client::get(&data.guid, &state.clients).await {
        Ok(Some(client)) => {
            client.jwt_request(state).await
        }
        Ok(None) => {
            let new_client = Client::new(data.guid.clone(), data.agent.clone());

            match new_client.insert(&state.clients).await {
                Ok(client) => client.jwt_request(state).await,
                Err(e) => HttpResponse::InternalServerError().json(doc! {
                    "error": format!("DB insert failed: {}", e)
                }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(doc! {
            "error": format!("DB lookup failed: {}", e)
        }),
    }
}
