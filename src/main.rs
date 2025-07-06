mod models;
mod admin;
mod client;
mod middleware;
use dotenv::dotenv;
use mongodb::Client;
use crate::models::app_state::AppState;
use crate::admin::AdminRoutes;
use actix_web::{web, App, HttpServer, get, Responder};
use std::env;
use std::io;
use bb8_redis::{bb8, RedisConnectionManager};
use crate::client::ClientRoutes;
use actix_web::middleware::Logger;
use env_logger::Env;
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
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let mongo_client = Client::with_uri_str(&mongodb_uri).await.unwrap();
    let db = mongo_client.database(&database_name);
    let redis_manager = RedisConnectionManager::new(redis_uri).unwrap();
    let redis_pool = bb8::Pool::builder()
        .max_size(15) // Adjust based on your needs
        .build(redis_manager)
        .await
        .unwrap();
    let state = web::Data::new(AppState {
        redis: redis_pool,
        clients: db.collection("clients"),
        files: db.collection("files"),
        admins: db.collection("admins"),
        jwt_secret
    });

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::Guardian)
            .app_data(state.clone())
            .service(health)
            .service(web::scope("/admin").configure(AdminRoutes::routes))
            .service(web::scope("/client").configure(ClientRoutes::routes))
    })
        .bind((ip.as_str(), port))?
        .run()
        .await
}