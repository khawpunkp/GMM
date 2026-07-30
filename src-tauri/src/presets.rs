use std::path::Path;

use rusqlite::{params, Connection};

use crate::models::Preset;
use crate::mods::{is_mod_enabled, toggle_mod};

pub fn list_presets(conn: &Connection) -> Result<Vec<Preset>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, is_favorite FROM presets ORDER BY name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Preset {
                id: row.get(0)?,
                name: row.get(1)?,
                is_favorite: row.get::<_, i64>(2)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

/// Records every mod's current on-disk enabled/disabled state against `preset_id` — presets are
/// full-system-state snapshots, not a chosen subset (matches the old app's `create_preset`/
/// `apply_preset` behavior).
fn snapshot_current_state(tx: &rusqlite::Transaction, base_mods_path: &Path, preset_id: i64) -> Result<(), String> {
    let all_mods: Vec<(i64, String)> = {
        let mut stmt = tx.prepare("SELECT id, folder_name FROM mods").map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
    };

    for (mod_id, folder_name) in all_mods {
        if let Some(enabled) = is_mod_enabled(base_mods_path, &folder_name) {
            tx.execute(
                "INSERT INTO preset_mods (preset_id, mod_id, is_enabled) VALUES (?1, ?2, ?3)
                 ON CONFLICT(preset_id, mod_id) DO UPDATE SET is_enabled = excluded.is_enabled",
                params![preset_id, mod_id, enabled as i64],
            )
            .map_err(|e| e.to_string())?;
        }
        // Mods missing from disk are silently skipped, same as the old app — nothing to snapshot.
    }
    Ok(())
}

pub fn create_preset(conn: &mut Connection, base_mods_path: &Path, name: &str) -> Result<Preset, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Preset name cannot be empty.".to_string());
    }

    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM presets WHERE LOWER(name) = LOWER(?1)",
            params![name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if existing > 0 {
        return Err(format!("Preset name '{}' already exists.", name));
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO presets (name) VALUES (?1)", params![name])
        .map_err(|e| e.to_string())?;
    let preset_id = tx.last_insert_rowid();

    snapshot_current_state(&tx, base_mods_path, preset_id)?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(Preset { id: preset_id, name: name.to_string(), is_favorite: false })
}

pub fn overwrite_preset(conn: &mut Connection, base_mods_path: &Path, preset_id: i64) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM preset_mods WHERE preset_id = ?1", params![preset_id])
        .map_err(|e| e.to_string())?;
    snapshot_current_state(&tx, base_mods_path, preset_id)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_preset(conn: &Connection, preset_id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM presets WHERE id = ?1", params![preset_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, preset_id: i64, is_favorite: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE presets SET is_favorite = ?1 WHERE id = ?2",
        params![is_favorite as i64, preset_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Renames whatever's out of sync with the preset's recorded state. `on_progress(processed, total,
/// current_mod_name)` fires per mod — kept as a plain closure (no Tauri `AppHandle`) for testability,
/// same pattern as the Phase 3 scanner's `run_scan`.
pub fn apply_preset(
    conn: &Connection,
    base_mods_path: &Path,
    preset_id: i64,
    mut on_progress: impl FnMut(usize, usize, &str),
) -> Result<String, String> {
    let entries: Vec<(i64, bool, String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT pm.mod_id, pm.is_enabled, m.folder_name, m.name
                 FROM preset_mods pm JOIN mods m ON pm.mod_id = m.id
                 WHERE pm.preset_id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![preset_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)? != 0,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
    };

    let total = entries.len();
    let mut errors = Vec::new();

    for (index, (mod_id, desired_enabled, folder_name, mod_name)) in entries.into_iter().enumerate() {
        on_progress(index + 1, total, &mod_name);

        match is_mod_enabled(base_mods_path, &folder_name) {
            Some(current_enabled) if current_enabled != desired_enabled => {
                if let Err(e) = toggle_mod(base_mods_path, &folder_name) {
                    errors.push(format!("Failed to toggle '{}' (ID {}): {}", mod_name, mod_id, e));
                }
            }
            Some(_) => {} // Already in the desired state.
            None => {
                errors.push(format!("Skipping '{}' (ID {}): folder not found on disk.", mod_name, mod_id));
            }
        }
    }

    if errors.is_empty() {
        Ok(format!("Successfully applied preset ({} mods processed).", total))
    } else {
        Err(format!("Preset applied with {} error(s):\n{}", errors.len(), errors.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mods::toggle_mod;
    use rusqlite::Connection;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn setup_test_db_and_dir() -> (Connection, PathBuf) {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::db::schema::SCHEMA).unwrap();

        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!("gmm_presets_test_{}_{}", std::process::id(), unique));
        let _ = fs::remove_dir_all(&base);

        for name in ["ModA", "ModB"] {
            fs::create_dir_all(base.join(name)).unwrap();
            fs::write(base.join(name).join("mod.ini"), "").unwrap();
            conn.execute(
                "INSERT INTO mods (name, folder_name) VALUES (?1, ?2)",
                params![name, name],
            )
            .unwrap();
        }

        (conn, base)
    }

    #[test]
    fn apply_preset_restores_snapshotted_state() {
        let (mut conn, base) = setup_test_db_and_dir();

        // Snapshot: both ModA and ModB enabled (their initial state).
        let preset = create_preset(&mut conn, &base, "My Preset").expect("create should succeed");

        // Disable ModA on disk, changing away from the snapshot.
        toggle_mod(&base, "ModA").unwrap();
        assert_eq!(is_mod_enabled(&base, "ModA"), Some(false));

        let mut progress_calls = Vec::new();
        let summary = apply_preset(&conn, &base, preset.id, |processed, total, name| {
            progress_calls.push((processed, total, name.to_string()));
        })
        .expect("apply should succeed");

        assert!(summary.contains("2 mods processed"));
        assert_eq!(progress_calls.len(), 2);
        // ModA should have been toggled back to enabled to match the snapshot.
        assert_eq!(is_mod_enabled(&base, "ModA"), Some(true));
        // ModB was never touched, so nothing should have changed for it.
        assert_eq!(is_mod_enabled(&base, "ModB"), Some(true));

        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn overwrite_preset_replaces_the_snapshot() {
        let (mut conn, base) = setup_test_db_and_dir();

        let preset = create_preset(&mut conn, &base, "My Preset").unwrap();
        toggle_mod(&base, "ModA").unwrap(); // ModA now disabled.

        overwrite_preset(&mut conn, &base, preset.id).expect("overwrite should succeed");

        // Applying now should be a no-op — the new snapshot already matches current disk state.
        apply_preset(&conn, &base, preset.id, |_, _, _| {}).unwrap();
        assert_eq!(is_mod_enabled(&base, "ModA"), Some(false));

        fs::remove_dir_all(&base).ok();
    }
}
