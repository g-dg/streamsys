use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::watch::{self};

use crate::database::slide_types::DbSlideType;

#[derive(Clone, Serialize, Deserialize)]
pub struct DisplayState {
    pub id: String,
    pub content: HashMap<String, String>,
    pub slide_type: Option<DbSlideType>,
}

impl DisplayState {
    pub fn new() -> Self {
        Self {
            id: String::default(),
            content: HashMap::new(),
            slide_type: None,
        }
    }
}

impl Default for DisplayState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DisplayStateService {
    pub watch_send: watch::Sender<DisplayState>,
    pub watch_recv: watch::Receiver<DisplayState>,
}

impl DisplayStateService {
    pub fn new() -> Self {
        let (send, recv) = watch::channel(DisplayState::default());

        Self {
            watch_send: send,
            watch_recv: recv,
        }
    }
}

impl Default for DisplayStateService {
    fn default() -> Self {
        Self::new()
    }
}
