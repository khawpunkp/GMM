use std::path::{Path, PathBuf};

use rusqlite::params;
use tauri::State;

use crate::keybinds;
use crate::models::PersistVar;
use crate::process_check::is_process_running;
use crate::DbState;

fn get_mods_folder(state: &State<DbState>) -> Result<PathBuf, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let path_str: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'mods_folder_path'", [], |row| row.get(0))
        .map_err(|_| "Mods folder path is not configured. Set it in Settings first.".to_string())?;
    Ok(PathBuf::from(path_str))
}

fn get_folder_name(state: &State<DbState>, mod_id: i64) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT folder_name FROM mods WHERE id = ?1", params![mod_id], |row| row.get(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_mod_persist_vars(mod_id: i64, state: State<DbState>) -> Result<Vec<PersistVar>, String> {
    let mods_path = get_mods_folder(&state)?;
    let folder_name = get_folder_name(&state, mod_id)?;
    Ok(keybinds::get_persist_vars(&mods_path, &folder_name))
}

#[tauri::command]
pub fn set_mod_persist_var(mod_id: i64, var_name: String, value: i64, state: State<DbState>) -> Result<(), String> {
    let mods_path = get_mods_folder(&state)?;
    let folder_name = get_folder_name(&state, mod_id)?;

    let game_path: Option<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row("SELECT value FROM settings WHERE key = 'game_executable_path'", [], |row| row.get(0))
            .ok()
    };
    if let Some(game_path) = game_path {
        if let Some(exe_name) = Path::new(&game_path).file_name().and_then(|n| n.to_str()) {
            if is_process_running(exe_name) {
                return Err(
                    "The game is currently running. Close it first — changing this while the game is \
                     open could be overwritten when the game exits."
                        .to_string(),
                );
            }
        }
    }

    keybinds::set_persist_var(&mods_path, &folder_name, &var_name, value)
}
