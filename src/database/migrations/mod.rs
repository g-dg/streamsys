use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

pub fn apply_migrations(
    conn: &PooledConnection<SqliteConnectionManager>,
    current_version: u32,
    latest_version: u32,
) {
    let _conn = conn;
    let _current_version = current_version;
    let _latest_version = latest_version;

    todo!();
}
