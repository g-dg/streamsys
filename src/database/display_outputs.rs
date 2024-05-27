use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DbDisplayOutput {
    pub id: Option<Uuid>,
    pub name: String,
    pub template_vue_script: Option<String>,
}
