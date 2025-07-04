mod client;
mod sessions;
mod commands;

use actix_web::{web};

pub struct AdminRoutes {}

impl AdminRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(client::fetch)
            .service(client::fetch_all)
            .service(sessions::fetch)
            .service(sessions::fetch_all);
    }
}