use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbSlideDeck {
    pub id: Option<Uuid>,
    pub name: String,
}
