use actix_web::{get, web, HttpResponse, Responder};
use bson::doc;
use bson::oid::ObjectId;
use crate::models::app_state::AppState;
use crate::models::jwt::Access;
use crate::models::session::Session;
#[get("/status/{id}")]
pub async fn fetch_status(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let client_id_str = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client_id"),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    match Session::status(&mut redis_conn, client_id, Access::Client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get status: {}", e)}),
    }
}