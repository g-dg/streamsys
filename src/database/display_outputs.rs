use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbDisplayOutput {
    pub id: Option<Uuid>,
    pub name: String,
}
