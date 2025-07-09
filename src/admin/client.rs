use actix_web::{Responder, get, HttpResponse, web};
use bson::oid::ObjectId;
use serde_json::json;
use crate::models::client::Client;
use crate::models::app_state::AppState;

#[get("/clients")]
pub async fn fetch_all(state: web::Data<AppState>) -> impl Responder {
    match Client::get_all(&state.clients).await {
        Ok(Some(clients)) => {HttpResponse::Ok().json(clients)}
        Ok(None) => {HttpResponse::Ok().json(json!([]))}
        Err(e) => HttpResponse::InternalServerError().json(e.to_string())
    }
}

#[get("/client/{id}")]
pub async fn fetch(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let client_id_str = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client id"),
    };

    match Client::get( &client_id, &state.clients).await {
        Ok(Some(client)) => HttpResponse::Ok().json(client),
        Ok(None) => {HttpResponse::Ok().json(json!([]))}
        Err(e) => HttpResponse::InternalServerError().json(e.to_string())
    }
}
