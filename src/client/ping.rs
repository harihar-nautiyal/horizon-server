use actix_web::{get, Responder};
use actix_web::web::Json;

#[get("/ping")]
pub async fn fetch() -> impl Responder {
    Json("{\"status\": \"pong\"}")
}