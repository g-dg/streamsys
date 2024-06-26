use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct DbSlideType {
    pub id: Option<Uuid>,
    pub name: String,
}
