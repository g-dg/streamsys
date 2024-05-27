use rusqlite::named_params;
use serde::{de::DeserializeOwned, Serialize};

use crate::database::Database;

pub struct ConfigService {
    db: Database,
}

impl ConfigService {
    pub fn new(database: &Database) -> Self {
        Self {
            db: database.clone(),
        }
    }

    pub fn get<T: Serialize + DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = self.db.get();

        let json = conn
            .prepare_cached("SELECT \"value_json\" FROM \"config\" WHERE \"key\" = :key;")
            .unwrap()
            .query_row(named_params! {":key": key}, |row| {
                row.get::<_, String>("value_json")
            })
            .expect("Error getting config value from database");

        serde_json::from_str(&json).expect("Error parsing JSON from config value")
    }

    pub fn set<T: Serialize + DeserializeOwned>(&self, key: &str, value: Option<&T>) {
        let conn = self.db.get();

        if let Some(value) = value {
            let json =
                serde_json::to_string(value).expect("Error stringifying JSON for config value");

            conn.prepare_cached(
                "INSERT INTO \"config\" (\"key\", \"value_json\") VALUES (:key, :value_json);",
            )
            .unwrap()
            .execute(named_params! {":key": key, ":value_json": json})
            .expect("Error setting config value in database");
        } else {
            conn.prepare_cached("DELETE FROM \"config\" WHERE \"key\" = :key;")
                .unwrap()
                .execute(named_params! {":key": key})
                .expect("Error clearing config value in database");
        }
    }
}
