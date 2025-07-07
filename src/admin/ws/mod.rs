mod uploads;
mod clients;
mod commands;

use actix_web::{rt, web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use serde_json::json;
use crate::models::commands::AdminCommand;
use crate::models::app_state::AppState;
use crate::models::command_result::CommandResult;

pub struct WebsocketsHandler {}

impl WebsocketsHandler {
    async fn handler(req: HttpRequest, stream: web::Payload, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
        let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

        let mut stream = stream
            .aggregate_continuations()
            .max_continuation_size(2_usize.pow(20));

        let state = state.clone();

        rt::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(AggregatedMessage::Text(text)) => {
                        match serde_json::from_str::<AdminCommand>(&text) {
                            Ok(command) => {
                                let response = match command.action.as_str() {
                                    // "ban" => ban::BanCommand::handle(&command.payload).await,
                                    // "status" => status::StatusCommand::handle(&command.payload).await,
                                    _ => CommandResult::Error {
                                        message: "Unknown action".to_string(),
                                    },
                                };

                                let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                                    json!({ "status": "error", "message": format!("Serialization error: {}", e) }).to_string()
                                });
                                if let Err(e) = session.text(response_json).await {
                                    log::error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                let error = json!({ "status": "error", "message": format!("Invalid command: {}", e) });
                                if let Err(e) = session.text(serde_json::to_string(&error).unwrap()).await {
                                    log::error!("Failed to send error: {}", e);
                                    break;
                                }
                            }
                        }
                    }

                    Ok(AggregatedMessage::Binary(bin)) => {
                        session.binary(bin).await.unwrap();
                    }

                    Ok(AggregatedMessage::Ping(msg)) => {
                        session.pong(&msg).await.unwrap();
                    }

                    _ => {}
                }
            }
        });

        Ok(res)
    }
}