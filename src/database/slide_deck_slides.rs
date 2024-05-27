use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbSlideDeckSlide {
    pub id: Option<Uuid>,
    pub slide_deck_section_id: Uuid,
    pub name_override: Option<String>,
    pub order: i64,
    pub slide_id: Option<Uuid>,
    pub slide_type_override_id: Option<Uuid>,
}
