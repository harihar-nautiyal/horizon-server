use serde::{Deserialize, Serialize};
use bson::{oid::ObjectId, DateTime};
use bb8_redis::{bb8::PooledConnection, RedisConnectionManager};
use redis::AsyncCommands;

#[derive(Deserialize, Serialize, Debug)]
pub struct Command {
    id: ObjectId,
    client: ObjectId,
    assigned_by: ObjectId,
    query: String,
    result: Option<String>,
    status: CommandStatus,
    registered_at: DateTime,
    resulted_at: Option<DateTime>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CommandStatus {
    Pending,
    Completed,
    Error,
    Canceled,
}

impl Command {
    pub fn new(client: ObjectId, assigned_by: ObjectId, query: String) -> Self {
        let now = DateTime::now();
        Self {
            id: ObjectId::new(),
            client,
            assigned_by,
            query,
            result: None,
            status: CommandStatus::Pending,
            registered_at: now.clone(),
            resulted_at: None,
        }
    }

    pub async fn register(
        &self,
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
    ) -> Result<(), String> {
        let command_key = format!("client:{}:commands", self.client);

        let command_json = serde_json::to_string(self)
            .map_err(|e| format!("Serialize error: {}", e))?;

        redis
            .rpush::<_, _, usize>(&command_key, command_json)
            .await
            .map_err(|e| format!("Redis R_PUSH error: {}", e))?;

        Ok(())
    }

    pub async fn update(
        &mut self,
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        result: String,
        status: CommandStatus,
    ) -> Result<(), String> {
        let command_key = format!("client:{}:commands", self.client);

        let commands: Vec<String> = redis
            .lrange(&command_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        if let Some((index, _)) = commands.iter().enumerate().find(|(_, cmd_str)| {
            serde_json::from_str::<Command>(cmd_str)
                .map(|c| c.id == self.id)
                .unwrap_or(false)
        }) {
            self.result = Some(result);
            self.status = status;
            self.resulted_at = Some(DateTime::now());

            let updated_json = serde_json::to_string(self)
                .map_err(|e| format!("Serialize error: {}", e))?;

            redis
                .lset::<_, _, ()>(&command_key, index as isize, updated_json)
                .await
                .map_err(|e| format!("Redis L_SET error: {}", e))?;

            Ok(())
        } else {
            Err("Command not found in Redis".into())
        }
    }

    pub async fn get(
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        client_id: ObjectId,
        command_id: ObjectId,
    ) -> Result<Option<Self>, String> {
        let command_key = format!("client:{}:commands", client_id);

        let commands: Vec<String> = redis
            .lrange(&command_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        for cmd_str in commands {
            if let Ok(cmd) = serde_json::from_str::<Self>(&cmd_str) {
                if cmd.id == command_id {
                    return Ok(Some(cmd));
                }
            }
        }

        Ok(None)
    }

    pub async fn get_all(
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        client_id: ObjectId,
    ) -> Result<Vec<Self>, String> {
        let command_key = format!("client:{}:commands", client_id);

        let commands: Vec<String> = redis
            .lrange(&command_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        let parsed = commands
            .into_iter()
            .filter_map(|s| serde_json::from_str::<Self>(&s).ok())
            .collect();

        Ok(parsed)
    }

    pub async fn get_pending(
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        client_id: ObjectId,
    ) -> Result<Vec<Self>, String> {
        let all = Self::get_all(redis, client_id).await?;
        let pending = all
            .into_iter()
            .filter(|cmd| matches!(cmd.status, CommandStatus::Pending))
            .collect();
        Ok(pending)
    }
}
