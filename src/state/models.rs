use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct CurrentState {
    pub id: String,
    pub display: DisplayState,
}

impl CurrentState {
    pub fn new() -> Self {
        Self {
            id: String::default(),
            display: DisplayState::new(),
        }
    }
}

impl Default for CurrentState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayState {
    pub content: HashMap<String, String>,
    pub slide_type_id: Option<Uuid>,
}

impl DisplayState {
    pub fn new() -> Self {
        Self {
            content: HashMap::new(),
            slide_type_id: None,
        }
    }
}

impl Default for DisplayState {
    fn default() -> Self {
        Self::new()
    }
}
