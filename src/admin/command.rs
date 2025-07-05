use actix_web::{get, Responder, post, put};

#[get("/command")]
async fn fetch_all() -> impl Responder {
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