pub mod schema;
pub mod seed;

use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

/// The mod-grouping feature (and `mod_group_members`) is brand new as of this version, so it's
/// always safe to drop and let `schema::SCHEMA` below recreate it with the `UNIQUE(mod_id)`
/// constraint — there is no prior release where a real mod group could have existed. Gated by a
/// one-time flag so this doesn't repeat (and destroy real group data) on every future startup.
fn migrate_mod_group_members_unique(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);")?;

    let already_migrated = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migration_mod_group_members_unique'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if !already_migrated {
        conn.execute("DROP TABLE IF EXISTS mod_group_members", [])?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('migration_mod_group_members_unique', 'true')",
            [],
        )?;
    }

    Ok(())
}

/// Presets were removed outright (not replaced by anything) — drop any existing data rather than
/// leaving orphaned tables `schema::SCHEMA` no longer recreates. Gated the same way as the group
/// migration above so this only runs once per install.
fn migrate_drop_presets(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);")?;

    let already_migrated = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migration_drop_presets'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if !already_migrated {
        conn.execute("DROP TABLE IF EXISTS preset_mods", [])?;
        conn.execute("DROP TABLE IF EXISTS presets", [])?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('migration_drop_presets', 'true')",
            [],
        )?;
    }

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The description field was removed from agents/mods entirely (not replaced by anything) — drop
/// the columns rather than leaving them as orphaned dead data. `column_exists` makes this a no-op
/// on a fresh install (the table doesn't exist yet, or already lacks the column). Gated the same
/// way as the migrations above so this only runs once per install.
fn migrate_drop_mod_agent_description(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);")?;

    let already_migrated = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'migration_drop_mod_agent_description'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .is_some();

    if !already_migrated {
        if column_exists(conn, "agents", "description")? {
            conn.execute("ALTER TABLE agents DROP COLUMN description", [])?;
        }
        if column_exists(conn, "mods", "description")? {
            conn.execute("ALTER TABLE mods DROP COLUMN description", [])?;
        }
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('migration_drop_mod_agent_description', 'true')",
            [],
        )?;
    }

    Ok(())
}

pub fn init_db(app_data_dir: &Path) -> rusqlite::Result<Connection> {
    std::fs::create_dir_all(app_data_dir).expect("failed to create app data dir");
    let db_path = app_data_dir.join("gmm.db");
    let conn = Connection::open(db_path)?;
    migrate_mod_group_members_unique(&conn)?;
    migrate_drop_presets(&conn)?;
    migrate_drop_mod_agent_description(&conn)?;
    conn.execute_batch(schema::SCHEMA)?;
    Ok(conn)
}
