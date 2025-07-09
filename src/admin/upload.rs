use actix_web::{get, Responder, post, put, web, HttpResponse, HttpRequest, HttpMessage};
use bson::doc;
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use crate::models::jwt::Claims;
use bson::oid::ObjectId;

use crate::models::upload::{Upload, Status};
#[derive(Deserialize, Serialize, Debug)]
struct PostUploadRequest {
    client: ObjectId,
    src: String
}

#[derive(Debug, Deserialize)]
pub struct UpdateUploadRequest {
    pub client_id: ObjectId,
    pub upload_id: ObjectId,
    pub status: Status,
    pub result: String,
}


#[get("/upload/{client_id}")]
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

    match Upload::get_all(&mut redis_conn, client_id).await {
        Ok(uploads) => HttpResponse::Ok().json(uploads),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}

#[get("/upload/{client_id}/pending")]
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

    match Upload::get_pending(&mut redis_conn, client_id).await {
        Ok(uploads) => HttpResponse::Ok().json(uploads),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}


#[get("/upload/{client_id}/{upload_id}")]
async fn fetch(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (client_id_str, upload_id_str) = path.into_inner();

    let client_id = match ObjectId::parse_str(&client_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid client_id"),
    };

    let upload_id = match ObjectId::parse_str(&upload_id_str) {
        Ok(oid) => oid,
        Err(_) => return HttpResponse::BadRequest().body("Invalid upload_id"),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(doc! {
                "error": format!("Failed to get Redis connection: {}", e)
            })
        }
    };

    match Upload::get(&mut redis_conn, client_id, upload_id).await {
        Ok(Some(command)) => HttpResponse::Ok().json(command),
        Ok(None) => HttpResponse::NotFound().body("Command not found"),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error: {}", err)),
    }
}

#[post("/upload")]
async fn create(state: web::Data<AppState>, data: web::Json<PostUploadRequest>, req: HttpRequest) -> impl Responder {
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

    let upload = Upload::new(data.client, admin_id.clone(), data.src.clone());

    match upload.register(&mut redis_conn).await {
        Ok(_) => HttpResponse::Ok().json(upload),
        Err(e) => HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)}),
    }
}

#[put("/upload")]
async fn update(
    state: web::Data<AppState>,
    data: web::Json<UpdateUploadRequest>,
) -> impl Responder {
    let UpdateUploadRequest {
        client_id,
        upload_id,
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

    let mut upload = match Upload::get(&mut redis_conn, client_id.clone(), upload_id.clone()).await {
        Ok(Some(cmd)) => cmd,
        Ok(None) => return HttpResponse::NotFound().body("Upload not found"),
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    match upload.update(&mut redis_conn, result, status).await {
        Ok(_) => HttpResponse::Ok().json(upload),
        Err(e) => HttpResponse::InternalServerError().body(format!("Update failed: {}", e)),
    }
}
