use actix_web::{post, Responder, web, HttpResponse};
use serde::{Deserialize};
use bson::{Bson, oid::ObjectId, doc};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Deserialize)]
struct RegisterRequest {
    guid: String,
    agent: String
}
//
// #[post("/register")]
// pub async fn register(state: web::Data<AppState>, data: web::Json<RegisterRequest>) -> impl Responder {
//     let object_id = match ObjectId::parse_str(&data.id) {
//         Ok(oid) => oid,
//         Err(_) => return HttpResponse::BadRequest().json(doc! {"error": "Invalid client ID format"}),
//     };
//
//     match state.clients.find_one(doc! { "_id": object_id }).await {
//         Ok(Some(_)) => (), // Client exists
//         Ok(None) => return HttpResponse::NotFound().json(doc! {"error": "Client not found"}),
//         Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)}),
//     };
//
//     let claims = Claims::new(data.id.clone(), chrono::Duration::hours(1)); // 1-hour expiration
//     let token = match encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
//     ) {
//         Ok(token) => token,
//         Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to generate token: {}", e)}),
//     };
//
//     HttpResponse::Ok().json(doc! {"token": token})
// }


