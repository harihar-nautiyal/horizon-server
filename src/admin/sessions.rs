use actix_web::{get, web, Responder};

#[get("/session")]
pub async fn fetch_all() -> impl Responder {
    "Sessions route".to_string()
}

#[get("/session/{id}")]
pub async fn fetch() -> impl Responder {
    "Session route".to_string()
}