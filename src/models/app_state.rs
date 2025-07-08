use mongodb::Collection;

use crate::models::client::Client;
use crate::models::admin::Admin;
use bb8_redis::{bb8, RedisConnectionManager};
pub struct AppState {
    pub redis: bb8::Pool<RedisConnectionManager>,
    pub clients: Collection<Client>,
    pub admins: Collection<Admin>,
    pub jwt_secret: String,
}