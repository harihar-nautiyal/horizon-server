use actix_web::{get, Responder, web, HttpResponse};
use actix_web::web::Json;
use bson::{doc, oid::ObjectId};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct PingRequest {
    id: String
}

#[get("/ping")]
pub async fn fetch(state: web::Data<AppState>, data: web::Json<PingRequest>) -> impl Responder {
    let object_id = match ObjectId::parse_str(&data.id) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json(doc! {"error": "Invalid client ID format"}),
    };

    let client = match state.clients.find_one(doc! { "_id": object_id }).await {
        Ok(Some(_)) => (),
        Ok(None) => return HttpResponse::NotFound().json(doc! {"error": "Client not found"}),
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)}),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    if let Err(e) = redis_conn.set::<String, String, ()>("pong".to_string(), "Hello world".to_string()).await {
        return HttpResponse::InternalServerError().json(doc! {"error": format!("Redis set failed: {}", e)});
    }

    let result: String = match redis_conn.get("pong").await {
        Ok(value) => value,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Redis get failed: {}", e)}),
    };

    HttpResponse::Ok().json(result)
}