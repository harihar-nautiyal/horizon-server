use actix_web::{get, Responder, post, put, web, HttpResponse};
use bson::doc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
#[derive(Deserialize, Serialize, Debug, Clone)]
struct CommandRequest {
    token: String
}
#[get("/command")]
async fn fetch_all(state: web::Data<AppState>, data: web::Json<CommandRequest>) -> impl Responder {

    let token_data = match decode::<Claims>(
        &data.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token) => token,
        Err(e) => return HttpResponse::Unauthorized().json(doc! { "error": format!("Invalid token: {}", e) }),
    };
    
    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    let command = format!("client:{}:status", client_id);
    
    "All commands here".to_string()
}

#[get("/command/{id}")]
async fn fetch() -> impl Responder {
    "Commands here".to_string()
}

#[post("/command")]
async fn create() -> impl Responder {
    "Post command here".to_string()
}

#[put("/command")]
async fn update() -> impl Responder {
    "Update command here".to_string()
}