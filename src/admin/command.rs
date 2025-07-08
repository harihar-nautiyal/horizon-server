use actix_web::{get, Responder, post, put, web, HttpResponse, HttpRequest, HttpMessage};
use bson::doc;
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
use bson::oid::ObjectId;

use crate::models::commands::{Command, CommandStatus};
#[derive(Deserialize, Serialize, Debug)]
struct PostCommandRequest {
    client: ObjectId,
    query: String
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommandRequest {
    pub client_id: ObjectId,
    pub command_id: ObjectId,
    pub status: CommandStatus,
    pub result: String,
}


#[get("/command/{client_id}")]
async fn fetch_all(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let client_id_str = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client_id"),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    match Command::get_all(&mut redis_conn, client_id).await {
        Ok(commands) => HttpResponse::Ok().json(commands),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}

#[get("/command/{client_id}/pending")]
async fn fetch_pending(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let client_id_str = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client_id"),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    match Command::get_pending(&mut redis_conn, client_id).await {
        Ok(commands) => HttpResponse::Ok().json(commands),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}


#[get("/command/{client_id}/{command_id}")]
async fn fetch(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (client_id_str, command_id_str) = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client_id"),
    };

    let command_id = match ObjectId::parse_str(&command_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid command_id"),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to get Redis connection: {}", e)
            })
        }
    };

    match Command::get(&mut redis_conn, client_id, command_id).await {
        Ok(Some(command)) => HttpResponse::Ok().json(command),
        Ok(None) => HttpResponse::NotFound().body("Command not found"),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}

#[post("/command")]
async fn create(state: web::Data<AppState>, data: web::Json<PostCommandRequest>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => {
            return HttpResponse::Unauthorized().json(doc! {
                "error": "Missing or invalid token (claims not found)"
            });
        }
    };

    let admin_id = &claims.id;

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    let command = Command::new(data.client, admin_id.clone(), data.query.clone());

    match command.register(&mut redis_conn).await {
        Ok(_) => HttpResponse::Ok().json(command),
        Err(e) => HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)}),
    }
}

#[put("/command")]
async fn update(
    state: web::Data<AppState>,
    data: web::Json<UpdateCommandRequest>,
) -> impl Responder {
    let UpdateCommandRequest {
        client_id,
        command_id,
        status,
        result,
    } = data.into_inner();

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to get Redis connection: {}", e)
            })
        }
    };

    let mut command = match Command::get(&mut redis_conn, client_id.clone(), command_id.clone()).await {
        Ok(Some(cmd)) => cmd,
        Ok(None) => return HttpResponse::NotFound().body("Command not found"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    match command.update(&mut redis_conn, result, status).await {
        Ok(_) => HttpResponse::Ok().json(command),
        Err(e) => HttpResponse::InternalServerError().body(format!("Update failed: {}", e)),
    }
}
