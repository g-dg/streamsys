use rusqlite::{named_params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    database::{users::DbUser, Database},
    helpers::errors::GenericError,
};

use super::auth::AuthService;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Uuid>,
    pub username: String,
    pub new_password: Option<String>,
    pub description: String,
    pub enabled: bool,
    pub permissions: i64,
}
impl User {
    pub fn from_db_user(user: &DbUser) -> Self {
        Self {
            id: Some(user.id),
            username: user.username.clone(),
            new_password: None,
            description: user.description.clone(),
            enabled: user.enabled,
            permissions: user.permissions,
        }
    }
}

pub struct UsersService {
    config: AppConfig,
    db: Database,
}

impl UsersService {
    pub fn new(database: &Database, config: &AppConfig) -> Self {
        Self {
            config: config.clone(),
            db: database.clone(),
        }
    }

    pub fn get_user_by_id(&self, id: Uuid) -> Option<DbUser> {
        let db = self.db.get();

        let user_result: Option<DbUser> = db
            .prepare_cached(&format!(
                "SELECT {} FROM \"users\" WHERE \"id\" = :id;",
                DbUser::COLUMNS_SQL
            ))
            .unwrap()
            .query_row(named_params! {":id": id}, |row| Ok(DbUser::from_row(row)))
            .optional()
            .expect("Error occurred getting user by id from database");

        user_result
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<DbUser> {
        let db = self.db.get();

        let user_result: Option<DbUser> = db
            .prepare_cached(&format!(
                "SELECT {} FROM \"users\" WHERE \"username\" = :username;",
                DbUser::COLUMNS_SQL
            ))
            .unwrap()
            .query_row(named_params! {":username": username}, |row| {
                Ok(DbUser::from_row(row))
            })
            .optional()
            .expect("Error occurred getting user by name from database");

        user_result
    }

    pub fn get(&self, id: Uuid) -> Option<User> {
        self.get_user_by_id(id)
            .map(|user| User::from_db_user(&user))
    }

    pub fn list(&self) -> Vec<User> {
        let db = self.db.get();

        let users = db
            .prepare_cached(&format!("SELECT {} FROM \"users\";", DbUser::COLUMNS_SQL))
            .unwrap()
            .query_map(named_params! {}, |row| Ok(DbUser::from_row(row)))
            .expect("Error occurred getting all users from database")
            .map(|db_user| User::from_db_user(&db_user.unwrap()))
            .collect();

        users
    }

    pub fn create(&self, user: &User) -> Result<Uuid, GenericError> {
        let user_id = Uuid::new_v4();

        let password_hash =
            AuthService::hash_password(&user.new_password.clone().unwrap_or_default());

        let db = self.db.get();
        let success = db.prepare_cached("INSERT INTO \"users\" (\"id\", \"username\", \"password\", \"description\", \"enabled\", \"permissions\") VALUES (:id, :username, :password, :description, :enabled, :permissions);")
            .unwrap()
            .execute(named_params! {
                ":id": user_id,
                ":username": user.username,
                ":password": password_hash,
                ":description": user.description,
                ":enabled": user.enabled,
                ":permissions": user.permissions,
            })
            .is_ok();

        if success {
            Ok(user_id)
        } else {
            Err(GenericError::BAD_REQUEST)
        }
    }

    pub fn update(&self, user: &User) -> Result<Uuid, GenericError> {
        let Some(user_id) = user.id else {
            return Err(GenericError::BAD_REQUEST);
        };
        let Some(old_user) = self.get_user_by_id(user_id) else {
            return Err(GenericError::NOT_FOUND);
        };

        let password_hash = if let Some(ref new_password) = user.new_password {
            AuthService::hash_password(new_password)
        } else {
            old_user.password
        };

        let db = self.db.get();
        let success = db.prepare_cached("UPDATE \"users\" SET \"username\" = :username, \"password\" = :password, \"description\" = :description, \"enabled\" = :enabled, \"permissions\" = :permissions WHERE \"id\" = :id;")
            .unwrap()
            .execute(named_params! {
                ":id": user.id,
                ":username": user.username,
                ":password": password_hash,
                ":description": user.description,
                ":enabled": user.enabled,
                ":permissions": user.permissions,
            })
            .is_ok();

        if success {
            Ok(user_id)
        } else {
            Err(GenericError::BAD_REQUEST)
        }
    }

    pub fn delete(&self, user_id: Uuid) -> Result<(), GenericError> {
        let Some(user) = self.get_user_by_id(user_id) else {
            return Err(GenericError::NOT_FOUND);
        };

        let db = self.db.get();

        let success = db
            .prepare_cached("DELETE FROM \"users\" WHERE \"id\" = :id;")
            .unwrap()
            .execute(named_params! {
                ":id": user.id,
            })
            .is_ok();

        if success {
            Ok(())
        } else {
            Err(GenericError::BAD_REQUEST)
        }
    }

    pub fn change_password(
        &self,
        user_id: Uuid,
        new_password: &str,
        session_to_keep: Option<&str>,
    ) -> Result<(), GenericError> {
        let password_hash = AuthService::hash_password(new_password);

        let db = self.db.get();
        let success = db
            .prepare_cached("UPDATE \"users\" SET \"password\" = :password WHERE \"id\" = :id;")
            .unwrap()
            .execute(named_params! {
                ":id": user_id,
                ":password": password_hash,
            })
            .is_ok();

        let auth_service = AuthService::new(&self.db, &self.config);
        auth_service.invalidate_sessions(user_id, session_to_keep);

        if success {
            Ok(())
        } else {
            Err(GenericError::BAD_REQUEST)
        }
    }
}
