use actix_web::{get, Responder, web, HttpResponse, HttpRequest, HttpMessage};
use bson::{doc};
use crate::models::app_state::AppState;
use crate::models::jwt::{Access, Claims};
use crate::models::session::Session;
use crate::models::client::Client;
use bson::Bson;
use crate::models::upload::Upload;
use crate::models::commands::Command;

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
        Ok(Some(client)) => client,
        Ok(None) => {
            return HttpResponse::NotFound().json(doc! {"error": "Client not found"});
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Database error: {}", e)
            });
        }
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to get Redis connection: {}", e)
            });
        }
    };

    // Update session activity
    if let Err(e) = Session::update_activity(&mut redis_conn, client_id, Access::Client).await {
        return HttpResponse::InternalServerError().json(e);
    }

    // Fetch pending uploads
    let pending_uploads = match Upload::get_pending(&mut redis_conn, *client_id).await {
        Ok(uploads) => uploads,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to fetch pending uploads: {}", e)
            });
        }
    };

    // Fetch pending commands
    let pending_commands = match Command::get_pending(&mut redis_conn, *client_id).await {
        Ok(commands) => commands,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to fetch pending commands: {}", e)
            });
        }
    };


    HttpResponse::Ok().json(doc! {
    "client": match bson::to_bson(&client) {
        Ok(Bson::Document(doc)) => doc,
        _ => return HttpResponse::InternalServerError().json(doc! { "error": "Failed to serialize client" }),
    },
    "pending_uploads": bson::to_bson(&pending_uploads).unwrap_or(Bson::Null),
    "pending_commands": bson::to_bson(&pending_commands).unwrap_or(Bson::Null)
})
}
