use actix_multipart::Multipart;
use actix_web::{web, post, Responder, HttpResponse, HttpRequest, HttpMessage};
use bson::doc;
use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;
use futures_util::StreamExt;
use crate::models::app_state::AppState;
use crate::models::upload::{Upload, Status};
use crate::models::jwt::Claims;
use sanitize_filename;

#[derive(Deserialize, Serialize, Debug)]
pub struct ResultUploadRequest {
    result: String
}
#[post("/upload/result/{id}")]
pub async fn result(
    state: web::Data<AppState>,
    mut payload: Multipart,
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    use std::io::Write;
    use uuid::Uuid;

    let extensions = req.extensions();
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => return HttpResponse::Unauthorized().json(doc! { "error": "Unauthorized" }),
    };

    let client_id = claims.id;
    let upload_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(doc! { "error": "Invalid ID" }),
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(doc! { "error": e.to_string() }),
    };

    let mut upload = match Upload::get(&mut redis_conn, client_id, upload_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::NotFound().json(doc! { "error": "Upload not found" }),
        Err(e) => return HttpResponse::InternalServerError().json(doc! { "error": e }),
    };

    let mut filename = None;

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().cloned();

        if let Some(cd) = content_disposition {
            if let Some(fname) = cd.get_filename() {
                let safe_filename = format!("{}_{}", Uuid::new_v4(), sanitize_filename::sanitize(fname));
                let filepath = format!("./uploads/{}", safe_filename);
                filename = Some(safe_filename.clone());

                let mut f = std::fs::File::create(&filepath).unwrap();
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    f.write_all(&data).unwrap();
                }
            }
        }
    }

    if let Some(file) = filename {
        if let Err(e) = upload.update(&mut redis_conn, file, Status::Uploaded).await {
            return HttpResponse::InternalServerError().json(doc! { "error": e });
        }

        HttpResponse::Ok().json(doc! {
            "success": true,
            "message": "File uploaded"
        })
    } else {
        HttpResponse::BadRequest().json(doc! {
            "error": "Missing file"
        })
    }
}
