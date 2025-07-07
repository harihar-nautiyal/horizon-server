use actix_web::{get, Responder, web, HttpResponse, HttpRequest, HttpMessage};
use bson::{doc};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
use crate::models::session::Session;
use crate::models::client::Client;



#[get("/ping")]
pub async fn pong(state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(doc! {
                "error": "Missing or invalid token (claims not found)"
            });
        }
    };

    let client_id = &claims.id;
    let client = match Client::get(client_id, &state.clients).await {
        Ok(Some(client)) => {
            client
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(doc! {"error": "Client not found"});
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)});
        }
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    match Session::update_activity(&mut redis_conn, client_id).await {
        Ok(_) => (),
        Err(e) => return HttpResponse::InternalServerError().json(e),
    }

    HttpResponse::Ok().json(client)
}