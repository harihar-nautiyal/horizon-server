use actix_web::{get, Responder, post, put, web, HttpResponse, HttpRequest, HttpMessage};
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
async fn fetch_all(state: web::Data<AppState>, data: web::Json<CommandRequest>, req: HttpRequest) -> impl Responder {
    let claims = req.extensions().get::<Claims>();
    

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