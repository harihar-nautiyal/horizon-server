use actix_web::{post, web, Error, HttpResponse};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use std::fs::File;
use std::io::Write;
use uuid::Uuid;

#[post("/upload")]
pub async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    while let Some(item) = payload.next().await {
        let mut field = item?;

        let content_disposition = field.content_disposition();
        let filename = content_disposition
            .get_filename()
            .map(|name| format!("uploads/{}_{}", Uuid::new_v4(), name))
            .unwrap_or_else(|| format!("uploads/{}", Uuid::new_v4()));

        let mut f = File::create(&filename)?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data)?;
        }

        println!("Uploaded to: {}", filename);
    }

    Ok(HttpResponse::Ok().json({ "status": "ok" }))
}
