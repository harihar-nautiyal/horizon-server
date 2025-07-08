use actix_web::{get, Responder, post, put, web, HttpResponse, HttpRequest, HttpMessage};
use bson::doc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
use bson::oid::ObjectId;
use crate::models::client::Client;
use crate::models::admin::Admin;
use crate::models::commands::Command;
#[derive(Deserialize, Serialize, Debug, Clone)]
struct FetchAllCommandRequest {
    client: ObjectId
}

#[derive(Deserialize, Serialize, Debug)]
struct PostCommandRequest {
    client: ObjectId,
    query: String
}

#[get("/command")]
async fn fetch_all(state: web::Data<AppState>, data: web::Json<FetchAllCommandRequest>, req: HttpRequest) -> impl Responder {
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
    let client = match Client::get(&data.client, &state.clients).await {
        Ok(Some(client)) => {
            client
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(doc! {"error": "admin not found"});
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)});
        }
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    match Command::get_all(&mut redis_conn, data.client).await {
        Ok(commands: Vec<Command>)
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
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)}),
    }
}

#[put("/command")]
async fn update() -> impl Responder {
    "Update command here".to_string()
}