use rusqlite::Row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
