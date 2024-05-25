use chrono::Utc;
use rusqlite::named_params;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::database::Database;

/// Manages auditing
pub struct AuditService {
    db: Database,
}

impl AuditService {
    pub fn new(database: &Database) -> Self {
        Self {
            db: database.clone(),
        }
    }

    pub fn log(&self, user_id: Option<Uuid>, action: &str) -> Uuid {
        self.internal_log::<()>(user_id, action, None)
    }

    pub fn log_data<T: Serialize + DeserializeOwned>(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        data: T,
    ) -> Uuid {
        self.internal_log(user_id, action, Some(data))
    }

    fn internal_log<T: Serialize + DeserializeOwned>(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        data: Option<T>,
    ) -> Uuid {
        let timestamp = Utc::now();
        let entry_id = Uuid::new_v4();

        let db = self.db.get();

        let serialized_data = data.map(|data| {
            serde_json::to_string(&data).expect("Error occurred serializing audit event data")
        });

        db.prepare_cached("INSERT INTO \"log\" (\"id\", \"user_id\", \"timestamp\", \"action\", \"data\") VALUES (:id, :user_id, :timestamp, :action, :data);")
            .unwrap()
            .execute(named_params! {
                ":id": entry_id,
                ":user_id": user_id,
                ":timestamp": timestamp,
                ":action": action,
                ":data": serialized_data
            })
            .expect("Error occurred while logging audit event");

        entry_id
    }
}
