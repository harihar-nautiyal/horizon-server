mod client;
mod sessions;
mod command;

use actix_web::{web};

pub struct AdminRoutes {}

impl AdminRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(client::fetch)
            .service(client::fetch_all)
            .service(sessions::fetch)
            .service(sessions::fetch_all)
            .service(command::fetch)
            .service(command::fetch_all)
            .service(command::update)
            .service(command::create);
    }
}