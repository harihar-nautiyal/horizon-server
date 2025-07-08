use actix_web::{web, post, Responder, HttpResponse, HttpRequest, HttpMessage};
use bson::doc;
use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use crate::models::app_state::AppState;
use crate::models::commands::{Command, CommandStatus};
use crate::models::jwt::Claims;

#[derive(Deserialize, Serialize, Debug)]
pub struct ResultCommandRequest {
    result: String
}
#[post("/commands/result/{id}")]
pub async fn result(
    state: web::Data<AppState>,
    data: web::Json<ResultCommandRequest>,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(doc! {
                "error": "Missing or invalid token (claims not found)"
            });
        }
    };

    let client_id = claims.id;
    let command_id_str = path.into_inner();

    let command_id = match ObjectId::parse_str(&command_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().json(doc! { "error": "Invalid command ID" }),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to get Redis connection: {}", e)
            });
        }
    };

    let mut command = match Command::get(&mut redis_conn, client_id, command_id).await {
        Ok(Some(cmd)) => cmd,
        Ok(None) => {
            return HttpResponse::NotFound().json(doc! {
                "error": "Command not found"
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to fetch command: {}", e)
            });
        }
    };

    let result = data.result.clone();

    if let Err(e) = command
        .update(&mut redis_conn, result, CommandStatus::Completed)
        .await
    {
        return HttpResponse::InternalServerError().json(doc! {
            "error": format!("Failed to update command: {}", e)
        });
    }

    HttpResponse::Ok().json(doc! {
        "success": true,
        "message": "Command marked as completed"
    })
}
