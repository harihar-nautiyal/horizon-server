mod models;
mod admin;
mod client;

use dotenv::dotenv;
use mongodb::Client;
use crate::models::app_state::AppState;
use crate::admin::AdminRoutes;
use actix_web::{web, App, HttpServer, get, Responder};
use std::env;
use std::io;
use redis;
use crate::client::ClientRoutes;

#[get("/health")]
async fn health() -> impl Responder {
    "Dont worry server is running fine".to_string()
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let ip = env::var("SERVER_IP").expect("SERVER_IP must be set");
    let port: u16 = env::var("SERVER_PORT")
        .expect("SERVER_PORT must be set")
        .parse()
        .expect("SERVER_PORT must be a number");
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let redis_uri = env::var("REDIS_URI").expect("REDIS_URI must be set");
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    let mongo_client = Client::with_uri_str(&mongodb_uri).await.unwrap();
    let redis_client = redis::Client::open(redis_uri).unwrap();
    let db = mongo_client.database(&database_name);

    let state = web::Data::new(AppState {
        redis: redis_client,
        clients: db.collection("clients"),
        files: db.collection("files"),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(health)
            .service(web::scope("/admin").configure(AdminRoutes::routes))
            .service(web::scope("/client").configure(ClientRoutes::routes))
    })
        .bind((ip.as_str(), port))?
        .run()
        .await
}