pub mod schema;
pub mod seed;

use rusqlite::Connection;
use std::path::Path;

pub fn init_db(app_data_dir: &Path) -> rusqlite::Result<Connection> {
    std::fs::create_dir_all(app_data_dir).expect("failed to create app data dir");
    let db_path = app_data_dir.join("gmm.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch(schema::SCHEMA)?;
    Ok(conn)
}
