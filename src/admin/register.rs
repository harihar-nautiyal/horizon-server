use actix_web::{post, web, HttpResponse, Responder};
use bson::{doc};
use serde::Deserialize;
use crate::models::app_state::AppState;
use crate::models::admin::Admin;

#[derive(Deserialize)]
struct RegisterRequest {
    guid: String,
    agent: String
}

#[post("/register")]
pub async fn register(state: web::Data<AppState>, data: web::Json<RegisterRequest>) -> impl Responder {
    match Admin::get_from_guid(&data.guid, &state.admins).await {
        Ok(Some(client)) => {
            client.jwt_request(state).await
        }
        Ok(None) => {
            let new_client = Admin::new(data.guid.clone(), data.agent.clone());

            match new_client.insert(&state.admins).await {
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
