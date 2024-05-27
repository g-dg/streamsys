use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbSlide {
    pub id: Option<Uuid>,
    pub slide_group_id: Option<Uuid>,
    pub slide_type_id: Option<Uuid>,
    pub name: String,
}
