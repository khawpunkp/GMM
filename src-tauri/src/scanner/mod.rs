pub mod archive;
pub mod deduce;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use walkdir::WalkDir;

use deduce::{deduce_mod_info, fetch_deduction_maps, has_ini_file, DISABLED_PREFIX};

/// Ports the old app's `scan_mods_directory`: walks the mods folder, identifies mod folders by
/// a non-excluded `.ini` file, fixes up a `DISABLED` -> `DISABLED_` naming inconsistency, runs the
/// deduction pipeline for new folders, then prunes DB rows for mods no longer found on disk.
///
/// `on_progress(processed_count, current_path)` fires once per mod folder found — kept as a plain
/// closure (no Tauri `AppHandle`) so this can run in a unit test as well as behind a real command.
pub fn run_scan(
    conn: &mut Connection,
    base_mods_path: &Path,
    mut on_progress: impl FnMut(usize, &str),
) -> Result<String, String> {
    if !base_mods_path.is_dir() {
        return Err(format!("Mods directory path is not a valid directory: {}", base_mods_path.display()));
    }

    let maps = fetch_deduction_maps(conn).map_err(|e| e.to_string())?;

    let mut found_folder_names = HashSet::<String>::new();
    let mut added = 0usize;
    let mut renamed = 0usize;
    let mut errors = 0usize;
    let mut processed = 0usize;

    let mut walker = WalkDir::new(base_mods_path).min_depth(1).into_iter();

    while let Some(entry_result) = walker.next() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[scan] error accessing entry: {}", e);
                errors += 1;
                continue;
            }
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let mut current_path = entry.path().to_path_buf();
        let filename = current_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        // Fix up a `DISABLED` (missing underscore) prefix before classifying the folder.
        if filename.starts_with("DISABLED") && !filename.starts_with(DISABLED_PREFIX) {
            let new_filename = format!("{}{}", DISABLED_PREFIX, filename.strip_prefix("DISABLED").unwrap_or(&filename));
            match current_path.parent() {
                Some(parent) => {
                    let new_path = parent.join(&new_filename);
                    match fs::rename(&current_path, &new_path) {
                        Ok(_) => {
                            current_path = new_path;
                            renamed += 1;
                        }
                        Err(e) => {
                            eprintln!("[scan] failed to rename '{}': {}", filename, e);
                            errors += 1;
                            walker.skip_current_dir();
                            continue;
                        }
                    }
                }
                None => {
                    errors += 1;
                    walker.skip_current_dir();
                    continue;
                }
            }
        }

        if !has_ini_file(&current_path) {
            continue; // Not a mod folder itself — let WalkDir descend into its children.
        }

        walker.skip_current_dir(); // Mod folders are leaves — don't look inside for nested mods.
        processed += 1;
        on_progress(processed, &current_path.display().to_string());

        let relative_path = match current_path.strip_prefix(base_mods_path) {
            Ok(p) => p,
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        let relative_filename = relative_path.file_name().unwrap_or_default().to_string_lossy();
        let clean_filename = relative_filename.strip_prefix(DISABLED_PREFIX).unwrap_or(&relative_filename);
        let clean_relative_path = match relative_path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join(clean_filename),
            _ => PathBuf::from(clean_filename),
        };
        let clean_relative_path_str = clean_relative_path.to_string_lossy().replace('\\', "/");

        found_folder_names.insert(clean_relative_path_str.clone());

        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM mods WHERE folder_name = ?1",
                params![clean_relative_path_str],
                |row| row.get(0),
            )
            .ok();

        if existing_id.is_none() {
            let deduced = deduce_mod_info(&current_path, base_mods_path, &maps);
            let insert_result = conn.execute(
                "INSERT INTO mods (agent_id, category_id, category_item_id, name, description, folder_name, image_filename, author)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    deduced.agent_id,
                    deduced.category_id,
                    deduced.category_item_id,
                    deduced.name,
                    deduced.description,
                    clean_relative_path_str,
                    deduced.image_filename,
                    deduced.author,
                ],
            );
            match insert_result {
                Ok(_) => added += 1,
                Err(e) => {
                    eprintln!("[scan] failed to insert mod '{}': {}", clean_relative_path_str, e);
                    errors += 1;
                }
            }
        }
    }

    let existing_folder_names: Vec<String> = {
        let mut stmt = conn.prepare("SELECT folder_name FROM mods").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| row.get(0)).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?
    };

    let mut pruned = 0usize;
    for folder_name in existing_folder_names {
        if !found_folder_names.contains(&folder_name) {
            conn.execute("DELETE FROM mods WHERE folder_name = ?1", params![folder_name])
                .map_err(|e| e.to_string())?;
            pruned += 1;
        }
    }

    Ok(format!(
        "Scan complete. Processed {} mod folders. Added {} new mods. Pruned {} missing mods. Renamed {} folders. {} errors.",
        processed, added, pruned, renamed, errors
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::db::schema::SCHEMA).unwrap();

        conn.execute("INSERT INTO agents (name, slug, is_builtin) VALUES ('Ellen', 'ellen', 1)", [])
            .unwrap();
        let ellen_id = conn.last_insert_rowid();
        for alias in ["ellen", "ellen joe", "ellenjoe", "joe"] {
            conn.execute(
                "INSERT INTO agent_aliases (agent_id, alias) VALUES (?1, ?2)",
                params![ellen_id, alias],
            )
            .unwrap();
        }

        conn.execute("INSERT INTO categories (name, slug) VALUES ('Enemies', 'enemies')", [])
            .unwrap();

        conn
    }

    /// Builds a fresh synthetic mods folder under the OS temp dir covering: agent match via
    /// parent folder name, category fallback match, DISABLED-prefix rename fixup (both the
    /// broken "DISABLED" and already-correct "DISABLED_" cases), an excluded-ini-only folder
    /// that must NOT be treated as a mod, and a folder that matches nothing at all.
    fn build_test_mods_dir() -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!("gmm_scanner_test_{}_{}", std::process::id(), unique));
        let _ = fs::remove_dir_all(&base);

        let ellen_skin = base.join("Ellen").join("EllenSkin_v2");
        fs::create_dir_all(&ellen_skin).unwrap();
        fs::write(ellen_skin.join("mod.ini"), "[Mod]\nName = My Ellen Skin\nAuthor = TestAuthor\n").unwrap();

        let enemy_reskin = base.join("Enemies").join("RandomReskin");
        fs::create_dir_all(&enemy_reskin).unwrap();
        fs::write(enemy_reskin.join("mod.ini"), "").unwrap();

        let broken_prefix = base.join("DISABLEDBrokenPrefix");
        fs::create_dir_all(&broken_prefix).unwrap();
        fs::write(broken_prefix.join("mod.ini"), "").unwrap();

        let already_correct = base.join("DISABLED_AlreadyCorrect");
        fs::create_dir_all(&already_correct).unwrap();
        fs::write(already_correct.join("mod.ini"), "").unwrap();

        let no_hints = base.join("RandomModWithNoHints");
        fs::create_dir_all(&no_hints).unwrap();
        fs::write(no_hints.join("mod.ini"), "").unwrap();

        let excluded_only = base.join("ExcludedOnly");
        fs::create_dir_all(&excluded_only).unwrap();
        fs::write(excluded_only.join("region.ini"), "").unwrap();

        base
    }

    #[test]
    fn scans_synthetic_mods_folder_correctly() {
        let mut conn = setup_test_db();
        let base = build_test_mods_dir();

        let summary = run_scan(&mut conn, &base, |_, _| {}).expect("scan should succeed");
        println!("{summary}");

        let (agent_id, name): (Option<i64>, String) = conn
            .query_row("SELECT agent_id, name FROM mods WHERE folder_name = 'Ellen/EllenSkin_v2'", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("Ellen mod should exist, matched via parent folder name");
        assert!(agent_id.is_some(), "Ellen mod should have matched an agent");
        assert_eq!(name, "My Ellen Skin", "mod name should come from the INI's Name field");

        let (agent_id2, category_id): (Option<i64>, Option<i64>) = conn
            .query_row("SELECT agent_id, category_id FROM mods WHERE folder_name = 'Enemies/RandomReskin'", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("Enemies mod should exist, matched via category fallback");
        assert!(agent_id2.is_none());
        assert!(category_id.is_some());

        let broken_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mods WHERE folder_name = 'BrokenPrefix'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(broken_count, 1, "DISABLED-prefixed folder should be renamed and stored under its clean name");
        assert!(base.join("DISABLED_BrokenPrefix").is_dir(), "folder should have been renamed with the underscore");
        assert!(!base.join("DISABLEDBrokenPrefix").is_dir(), "old incorrectly-prefixed folder should no longer exist");

        let already_correct_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mods WHERE folder_name = 'AlreadyCorrect'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(already_correct_count, 1, "already-correctly-prefixed DISABLED_ folder should scan without a rename");

        let excluded_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM mods WHERE folder_name LIKE '%ExcludedOnly%'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(excluded_count, 0, "a folder containing only an excluded .ini (region.ini) must not be treated as a mod");

        let (agent_id3, category_id3): (Option<i64>, Option<i64>) = conn
            .query_row("SELECT agent_id, category_id FROM mods WHERE folder_name = 'RandomModWithNoHints'", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("uncategorized mod should still be recorded");
        assert!(agent_id3.is_none());
        assert!(category_id3.is_none());

        fs::remove_dir_all(&base).ok();
    }
}
