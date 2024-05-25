pub mod migrations;
pub mod users;

use std::{thread, time::Duration};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use serde_json::json;

use crate::{
    config::AppConfig,
    services::{
        audit::AuditService,
        users::{User, UsersService},
    },
};

use self::users::UserPermission;

const DATABASE_DEFINITION_SQL: &str = include_str!("../../database.sql");

const DATABASE_VERSION_MIN: u32 = 1;
const DATABASE_VERSION_MAX: u32 = 999;

const OPTIMIZE_INCREMENTAL_VACUUM_PAGES: u64 = 1;

#[derive(Clone)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    /// Creates a new database connection pool
    pub fn new(config: &AppConfig) -> Self {
        let manager = SqliteConnectionManager::file(&config.database_file).with_init(|c| {
            // set pragmas
            c.execute_batch(
                "PRAGMA busy_timeout = 60000; \
                PRAGMA journal_mode = WAL; \
                PRAGMA synchronous = NORMAL; \
                PRAGMA foreign_keys = 1; \
                PRAGMA auto_vacuum = INCREMENTAL; \
                PRAGMA recursive_triggers = 1; \
                PRAGMA wal_autocheckpoint = 0;",
            )
        });

        let pool = r2d2::Pool::new(manager).expect("Error occurred connecting to database");
        let conn = pool
            .get()
            .expect("Error occurred getting database connection for initialization");

        let db = Self { pool };

        let version: u32 = conn
            .prepare("PRAGMA user_version;")
            .unwrap()
            .query_row([], |row| row.get(0))
            .expect("Error occurred getting database user version");

        if version == 0 {
            // create database if not yet created
            conn.execute_batch(DATABASE_DEFINITION_SQL)
                .expect("Error occurred while running database initialization commands");

            let audit_service = AuditService::new(&db);

            audit_service.log(None, "init");

            let user_service = UsersService::new(&db, config);

            // add default admin user
            let user_id = user_service
                .create(&User {
                    id: None,
                    username: String::from(&config.default_admin_username),
                    new_password: Some(String::from(&config.default_admin_password)),
                    description: String::new(),
                    enabled: true,
                    permissions: UserPermission::MODIFY_SELF | UserPermission::USER_ADMIN,
                })
                .expect("Error occurred creating default admin user");

            audit_service.log_data(None, "default_user_created", json!({"user_id": user_id}));
        } else if version > DATABASE_VERSION_MAX {
            // panic of database is too new (i.e. unsupported)
            panic!("Database version is too new. Please check for application updates.");
        } else if version < DATABASE_VERSION_MIN {
            // apply migrations if older than latest database version
            migrations::apply_migrations(&conn, version, DATABASE_VERSION_MIN);
        }

        // run full database optimization
        db.optimize(true);

        // run full checkpoint
        db.checkpoint(true);

        db
    }

    /// Gets an instance of the database connection pool
    pub fn get(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool
            .get()
            .expect("Error occurred getting database connection from connection pool")
    }

    /// Runs a database checkpoint
    pub fn checkpoint(&self, full: bool) {
        let conn = self
            .pool
            .get()
            .expect("Error occurred getting database connection for checkpoint");

        if full {
            let mut stmt = conn.prepare("PRAGMA wal_checkpoint(RESTART);").unwrap();
            while stmt
                .query_row([], |row| row.get::<usize, i32>(0))
                .expect("Error occurred while checkpointing database")
                == 1
            {
                // busy wait for successful checkpoint
                thread::sleep(Duration::from_millis(1));
            }
        } else {
            conn.execute_batch("PRAGMA wal_checkpoint;")
                .expect("Error occurred while checkpointing database");
        };
    }

    /// Runs database optimization tasks
    pub fn optimize(&self, full: bool) {
        let conn = self
            .pool
            .get()
            .expect("Error occurred getting database connection for optimization");

        if full {
            conn.execute_batch(
                "PRAGMA optimize = 0x10002; \
                PRAGMA incremental_vacuum;",
            )
            .ok();
        } else {
            conn.execute_batch(&format!(
                "PRAGMA optimize = 0x10012; \
                PRAGMA incremental_vacuum({OPTIMIZE_INCREMENTAL_VACUUM_PAGES});"
            ))
            .ok();
        }
    }

    /// Checks database integrity
    pub fn integrity_check(&self) -> bool {
        let conn = self
            .pool
            .get()
            .expect("Error occurred getting database connection for integrity checks");

        if conn
            .prepare("PRAGMA integrity_check(1);")
            .unwrap()
            .query_row([], |row| Ok(row.get::<usize, String>(0)))
            .expect("Error occurred while checking database integrity")
            .unwrap()
            != *"ok"
        {
            return false;
        }

        if conn
            .prepare("PRAGMA foreign_key_check;")
            .unwrap()
            .query_map([], |row| Ok(row.get::<usize, String>(0)))
            .expect("Error occurred while checking database foreign key integrity")
            .count()
            > 0
        {
            return false;
        }

        true
    }
}
