mod client;
mod sessions;
mod command;
mod register;
mod ping;
mod middleware;
mod ws;

use actix_web::{web, HttpRequest, HttpResponse, Error, rt};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use crate::models::commands::AdminCommand;

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
            .service(command::create)
            .service(register::register);
    }
}