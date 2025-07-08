mod client;
mod sessions;
mod command;
mod register;
mod ping;
mod ws;
mod uploads;

use actix_web::{web};

pub struct AdminRoutes {}

impl AdminRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(client::fetch)
            .service(client::fetch_all)
            .service(sessions::fetch_status)
            .service(command::fetch)
            .service(command::fetch_all)
            .service(command::update)
            .service(command::create)
            .service(register::register)
            .service(ping::pong);
    }
}