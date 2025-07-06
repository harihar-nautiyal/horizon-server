use actix_web::{get, Responder, web, HttpResponse};
use bson::{doc, oid::ObjectId};
use redis::{AsyncCommands};
use serde::{Deserialize, Serialize};
use crate::models::app_state::AppState;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::models::jwt::Claims;
use crate::models::session::Session;

const PING_INTERVAL: i64 = 10;
const STATUS_TTL: i64 = 15;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct PingRequest {
    token: String
}

#[get("/ping")]
pub async fn pong(state: web::Data<AppState>, data: web::Json<PingRequest>) -> impl Responder {

    let token_data = match decode::<Claims>(
        &data.token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token) => token,
        Err(e) => return HttpResponse::Unauthorized().json(doc! { "error": format!("Invalid token: {}", e) }),
    };

    let admin_id = &token_data.claims.id;

    println!("Admin ID from token: {}", admin_id);

    let object_id = match ObjectId::parse_str(&admin_id) {
        Ok(oid) => oid,
        Err(e) => {
            println!("Failed to parse ObjectId '{}': {}", admin_id, e);
            return HttpResponse::BadRequest().json(doc! {"error": "Invalid client ID format"});
        }
    };

    println!("Parsed ObjectId: {}", object_id);

    match state.admins.find_one(doc! { "_id": object_id }).await {
        Ok(Some(admin)) => {
            println!("Found client: {:?}", admin);
            admin
        },
        Ok(None) => {
            println!("Admin not found with ObjectId: {}", object_id);
            return HttpResponse::NotFound().json(doc! {"error": "Admin not found"});
        },
        Err(e) => {
            println!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(doc! {"error": format!("Database error: {}", e)});
        }
    };

    let mut redis_conn = match state.redis.get().await {
        Ok(conn) => conn,
        Err(e) => return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to get Redis connection: {}", e)}),
    };

    let status_key = format!("admin:{}:status", admin_id);
    let sessions_key = format!("admin:{}:sessions", admin_id);
    let last_ping_key = format!("admin:admin_id{}:lastPing", admin_id);

    let now = chrono::Utc::now();
    let now_ms = now.timestamp_millis();
    let now_iso = now.to_rfc3339();

    let is_active: Option<String> = redis_conn.get(&status_key).await.unwrap_or(None);

    if is_active.is_none() {
        let session = Session {
            start: now_iso.clone(),
            end: None,
            start_ms: now_ms,
            end_ms: None,
        };
        if let Err(e) = redis_conn.rpush::<_, _, usize>(&sessions_key, serde_json::to_string(&session).unwrap()).await {
            return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to store session: {}", e)});
        }
        println!("Admin {} started new session at {}", admin_id, now_iso);
    }

    let last_session: Option<String> = match redis_conn.lindex::<_, Option<String>>(&sessions_key, -1).await {
        Ok(value) => value,
        Err(_) => None,
    };
    if let Some(session_str) = last_session {
        let mut session: Session = serde_json::from_str(&session_str).unwrap();
        if session.end.is_none() && is_active.is_none() {
            session.end = Some(now_iso.clone());
            session.end_ms = Some(now_ms);
            if let Err(e) = redis_conn.lset::<_, _, ()>(&sessions_key, -1, serde_json::to_string(&session).unwrap()).await {
                return HttpResponse::InternalServerError().json(doc! {"error": format!("Failed to update session: {}", e)});
            }
            println!("Admin {} session ended at {}", admin_id, now_iso);
        }
    }

    if let Err(e) = redis_conn.set_ex::<_, _, ()>(status_key, "active", STATUS_TTL as u64).await {
        return HttpResponse::InternalServerError().json(doc! {"error": format!("Redis set failed: {}", e)});
    }
    if let Err(e) = redis_conn.set_ex::<_, _, ()>(last_ping_key, now_iso, STATUS_TTL as u64).await {
        return HttpResponse::InternalServerError().json(doc! {"error": format!("Redis set failed: {}", e)});
    }

    HttpResponse::Ok().json(doc! {"status": "pong"})
}