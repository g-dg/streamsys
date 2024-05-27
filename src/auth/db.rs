use chrono::{DateTime, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct UserPermission {}
impl UserPermission {
    pub const ANY: i64 = -1;
    pub const MODIFY_SELF: i64 = 1 << 0;
    pub const USER_ADMIN: i64 = 1 << 1;
    pub const SYSTEM_ADMIN: i64 = 1 << 2;
    pub const SETUP: i64 = 1 << 3;
    pub const OPERATION: i64 = 1 << 4;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DbSession {
    pub id: Uuid,
    pub token: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub valid: bool,
}
impl DbSession {
    pub const TABLE_NAME: &'static str = "sessions";

    pub const COLUMNS_SQL: &'static str =
        "\"id\", \"token\", \"user_id\", \"timestamp\", \"valid\"";

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get("id")
                .expect("Failed to get value from database row"),
            token: row
                .get("token")
                .expect("Failed to get value from database row"),
            user_id: row
                .get("user_id")
                .expect("Failed to get value from database row"),
            timestamp: row
                .get("timestamp")
                .expect("Failed to get value from database row"),
            valid: row
                .get("valid")
                .expect("Failed to get value from database row"),
        }
    }
}
