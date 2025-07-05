mod ping;
mod upload;
mod result;
mod register;

use actix_web::web;

pub struct ClientRoutes {}

impl ClientRoutes {
    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg
            .service(ping::fetch)
            .service(upload::fetch)
            .service(result::fetch);
    }
}