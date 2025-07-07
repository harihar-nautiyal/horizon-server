use serde::Serialize;
use crate::models::command_result::CommandResult;

pub mod list {
    use serde_json::json;
    use super::CommandResult;

    pub struct ListCommand;

    impl ListCommand {
        pub async fn handle(payload: &serde_json::Value) -> CommandResult {
            let username = match payload.get("username") {
                Some(value) => value.as_str().unwrap_or(""),
                None => return CommandResult::Error { message: "Missing username".to_string() },
            };

            // Simulate banning the user (replace with real logic, e.g., database update)
            println!("Banning user: {}", username);

            CommandResult::Success {
                data: json!({ "message": format!("User {} banned", username) })
            }
        }
    }
}