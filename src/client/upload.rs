use actix_web::{get, Responder};
use actix_web::web::Json;

#[get("/upload")]
pub async fn fetch() -> impl Responder {
    Json("{\"status\": \"uploading\"}")
}