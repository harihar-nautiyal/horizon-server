use mongodb::Collection;

use crate::models::client::Client;
use crate::models::file::File;
pub struct AppState {
    pub redis: redis::Client,
    pub clients: Collection<Client>,
    pub files: Collection<File>,
}