mod ping;
mod register;
mod commands;
mod upload;

use actix_web::web;

pub struct ClientRoutes {}

impl ClientRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(ping::pong)
            .service(commands::result)
            .service(register::register);
    }
}