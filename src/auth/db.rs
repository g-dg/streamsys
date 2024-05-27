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
pub struct DbUser {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub description: String,
    pub enabled: bool,
    pub permissions: i64,
}
impl DbUser {
    pub const TABLE_NAME: &'static str = "users";

    pub const COLUMNS_SQL: &'static str =
        "\"id\", \"username\", \"password\", \"enabled\", \"description\", \"permissions\"";

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get("id")
                .expect("Failed to get value from database row"),
            username: row
                .get("username")
                .expect("Failed to get value from database row"),
            password: row
                .get("password")
                .expect("Failed to get value from database row"),
            description: row
                .get("description")
                .expect("Failed to get value from database row"),
            enabled: row
                .get("enabled")
                .expect("Failed to get value from database row"),
            permissions: row
                .get("permissions")
                .expect("Failed to get value from database row"),
        }
    }
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
