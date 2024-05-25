use std::io::ErrorKind;

use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,

    #[serde(default = "default_database_file")]
    pub database_file: String,

    #[serde(default = "default_default_admin_username")]
    pub default_admin_username: String,

    #[serde(default = "default_default_admin_password")]
    pub default_admin_password: String,

    #[serde(default = "default_session_max_age")]
    pub session_max_age: u64,

    #[serde(default = "default_static_file_root")]
    pub static_file_root: String,

    #[serde(default = "default_static_file_index")]
    pub static_file_index: String,

    #[serde(default = "default_http_caching_max_age")]
    pub http_caching_max_age: u64,

    #[serde(default = "default_database_maintenance_interval")]
    pub database_maintenance_interval: u64,

    #[serde(default = "default_database_quick_checkpoint_interval")]
    pub database_quick_checkpoint_interval: u64,

    #[serde(default = "default_database_full_checkpoint_interval")]
    pub database_full_checkpoint_interval: u64,
}

impl AppConfig {
    pub async fn load(filename: &str) -> Self {
        let contents = match fs::read_to_string(filename).await {
            Ok(value) => value,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    String::from("{}")
                } else {
                    panic!(
                        "Error occurred reading config file \"{}\": {:?}",
                        filename, err
                    )
                }
            }
        };

        serde_json::from_str(&contents).expect("Error occurred parsing config file")
    }
}

fn default_host() -> String {
    String::from("127.0.0.1")
}
fn default_port() -> u16 {
    8080
}
fn default_cors_allowed_origins() -> Vec<String> {
    Vec::from([
        // String::from("http://localhost:5173"),
        // String::from("http://127.0.0.1:5173"),
    ])
}
fn default_database_file() -> String {
    String::from("./database.sqlite3")
}
fn default_default_admin_username() -> String {
    String::from("admin")
}
fn default_default_admin_password() -> String {
    String::from("admin")
}
fn default_session_max_age() -> u64 {
    60 * 60 * 24 * 365
}
fn default_static_file_root() -> String {
    String::from("./client/dist/")
}
fn default_static_file_index() -> String {
    String::from("index.html")
}
fn default_http_caching_max_age() -> u64 {
    60 * 60 * 24
}
fn default_database_maintenance_interval() -> u64 {
    60 * 60
}
fn default_database_quick_checkpoint_interval() -> u64 {
    60
}
fn default_database_full_checkpoint_interval() -> u64 {
    60 * 5
}
