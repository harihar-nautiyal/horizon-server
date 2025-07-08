use bb8_redis::bb8::PooledConnection;
use bb8_redis::RedisConnectionManager;
use serde::{Serialize, Deserialize};
use bson::{DateTime, oid::ObjectId};
use redis::AsyncCommands;

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    pub id: ObjectId,
    pub client: ObjectId,
    pub assigned_by: ObjectId,
    pub src_file: String,
    pub download_file: Option<String>,
    pub status: Status,
    pub registered_at: DateTime,
    pub uploaded_at: Option<DateTime>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Uploaded,
    Cancelled,
    Uploading
}

impl Upload {
    pub fn new(client: ObjectId, assigned_by: ObjectId, src_file: String) -> Upload {
        let now = DateTime::now();
        Upload {
            id: ObjectId::new(),
            client,
            assigned_by,
            src_file,
            download_file: None,
            status: Status::Pending,
            registered_at: now.clone(),
            uploaded_at: None
        }
    }

    pub async fn register(
        &self,
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
    ) -> Result<(), String> {
        let upload_key = format!("client:{}:uploads", self.client);

        let upload_json = serde_json::to_string(self)
            .map_err(|e| format!("Serialize error: {}", e))?;

        redis
            .rpush::<_, _, usize>(&upload_key, upload_json)
            .await
            .map_err(|e| format!("Redis R_PUSH error: {}", e))?;

        Ok(())
    }

    pub async fn update(
        &mut self,
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        download: String,
        status: Status,
    ) -> Result<(), String> {
        let upload_key = format!("client:{}:uploads", self.client);

        let uploads: Vec<String> = redis
            .lrange(&upload_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        if let Some((index, _)) = uploads.iter().enumerate().find(|(_, cmd_str)| {
            serde_json::from_str::<Self>(cmd_str)
                .map(|c| c.id == self.id)
                .unwrap_or(false)
        }) {
            self.download_file = Some(download);
            self.status = status;
            self.uploaded_at = Some(DateTime::now());

            let updated_json = serde_json::to_string(self)
                .map_err(|e| format!("Serialize error: {}", e))?;

            redis
                .lset::<_, _, ()>(&upload_key, index as isize, updated_json)
                .await
                .map_err(|e| format!("Redis L_SET error: {}", e))?;

            Ok(())
        } else {
            Err("Upload not found in Redis".into())
        }
    }

    pub async fn get(
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        client_id: ObjectId,
        upload_id: ObjectId,
    ) -> Result<Option<Self>, String> {
        let upload_key = format!("client:{}:uploads", client_id);

        let uploads: Vec<String> = redis
            .lrange(&upload_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        for upl_str in uploads {
            if let Ok(upl) = serde_json::from_str::<Self>(&upl_str) {
                if upl.id == upload_id {
                    return Ok(Some(upl));
                }
            }
        }

        Ok(None)
    }

    pub async fn get_all(
        redis: &mut PooledConnection<'_, RedisConnectionManager>,
        upload_id: ObjectId,
    ) -> Result<Vec<Self>, String> {
        let upload_key = format!("client:{}:uploads", upload_id);

        let uploads: Vec<String> = redis
            .lrange(&upload_key, 0, -1)
            .await
            .map_err(|e| format!("Redis L_RANGE error: {}", e))?;

        let parsed = uploads
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
            .filter(|cmd| matches!(cmd.status, Status::Pending))
            .collect();
        Ok(pending)
    }
}