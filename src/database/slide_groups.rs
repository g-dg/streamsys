use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbSlideGroup {
    pub id: Option<Uuid>,
    pub parent_group_id: Option<Uuid>,
    pub name: String,
}
