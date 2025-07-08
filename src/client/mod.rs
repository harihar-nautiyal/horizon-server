mod ping;
mod result;
mod register;

use actix_web::web;

pub struct ClientRoutes {}

impl ClientRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(ping::pong)
            .service(result::fetch)
            .service(register::register);
    }
}