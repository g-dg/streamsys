use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbSlideDeck {
    pub id: Option<Uuid>,
    pub slide_deck_id: Uuid,
    pub name: Option<String>,
    pub order: i64,
    pub slide_group_id: Option<Uuid>,
    pub slide_type_override_id: Option<Uuid>,
}
