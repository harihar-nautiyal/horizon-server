use actix_web::{Responder, get};

#[get("/client")]
pub async fn fetch_all() -> impl Responder {
    "Clients route".to_string()
}

#[get("/client/{id}")]
pub async fn fetch() -> impl Responder {
    "Client route".to_string()
}
