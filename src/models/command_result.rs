use serde::Serialize;

#[derive(Serialize)]
pub enum CommandResult {
    Success { data: serde_json::Value },
    Error { message: String },
}
