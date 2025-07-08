use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use bb8_redis::{bb8::PooledConnection, RedisConnectionManager};
use bson::doc;
use bson::oid::ObjectId;
use crate::models::jwt::Access;
use crate::models::client::Client;
use crate::models::admin::Admin;

const STATUS_TTL: i64 = 15;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    pub start_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_ms: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Offline
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Record {
    Admin(Admin),
    Client(Client)
}

impl Session {
    pub async fn update_activity(
        redis:  &mut PooledConnection<'_, RedisConnectionManager>,
        client_id: &ObjectId,
        access: Access
    ) -> Result<(), String> {

        let access_str = access.to_string();
        let status_key = format!("{}:{}:status", access_str, client_id);
        let sessions_key = format!("{}:{}:sessions", access_str, client_id);
        let last_ping_key = format!("{}:{}:lastPing",access_str, client_id);

        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let now_iso = now.to_rfc3339();

        let is_active: Option<String> = redis.get(&status_key).await.unwrap_or(None);

        if is_active.is_none() {
            let session = Session {
                start: now_iso.clone(),
                end: None,
                start_ms: now_ms,
                end_ms: None,
            };

            let session_json = serde_json::to_string(&session)
                .map_err(|e| format!("Serialize error: {}", e))?;

            redis
                .rpush::<_, _, usize>(&sessions_key, session_json)
                .await
                .map_err(|e| format!("Redis R_PUSH error: {}", e))?;
        }

        if let Ok(Some(session_str)) = redis.lindex::<_, Option<String>>(&sessions_key, -1).await {
            if let Ok(mut session) = serde_json::from_str::<Session>(&session_str) {
                if session.end.is_none() && is_active.is_none() {
                    session.end = Some(now_iso.clone());
                    session.end_ms = Some(now_ms);

                    let updated = serde_json::to_string(&session)
                        .map_err(|e| format!("Serialize error: {}", e))?;

                    redis
                        .lset::<_, _, ()>(&sessions_key, -1, updated)
                        .await
                        .map_err(|e| format!("Redis L_SET error: {}", e))?;
                }
            }
        }

        redis
            .set_ex::<_, _, ()>(&status_key, "active", STATUS_TTL as u64)
            .await
            .map_err(|e| format!("Redis SET_EX status error: {}", e))?;

        redis
            .set_ex::<_, _, ()>(&last_ping_key, now_iso, STATUS_TTL as u64)
            .await
            .map_err(|e| format!("Redis SET_EX last ping error: {}", e))?;

        Ok(())
    }

    pub async fn status(redis:  &mut PooledConnection<'_, RedisConnectionManager>, id: ObjectId, access: Access) -> Result<Status, String> {
        let status_key = format!("{}:{}:status", access.to_string(), id);

        match redis.get::<_, Option<String>>(status_key).await {
            Ok(Some(status_str)) if status_str == "active" => Ok(Status::Online),
            Ok(_) => Ok(Status::Offline),
            Err(e) => Err(format!("Redis GET status error: {}", e)),
        }
    }
}
