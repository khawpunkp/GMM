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

fn game_is_running(state: &State<DbState>) -> Result<bool, String> {
    let game_path: Option<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row("SELECT value FROM settings WHERE key = 'game_executable_path'", [], |row| row.get(0))
            .ok()
    };
    Ok(game_path
        .as_deref()
        .and_then(|p| Path::new(p).file_name().and_then(|n| n.to_str()))
        .map(is_process_running)
        .unwrap_or(false))
}

/// Lets the frontend show the persist-var editor as read-only while the game is running, instead
/// of only finding out via a failed write — 3DMigoto owns that value while the game is up (it
/// writes it back into the ini on reload/exit), so an app-side edit would just get overwritten.
#[tauri::command]
pub fn is_game_running(state: State<DbState>) -> Result<bool, String> {
    game_is_running(&state)
}

#[tauri::command]
pub fn set_mod_persist_var(mod_id: i64, var_name: String, value: i64, state: State<DbState>) -> Result<(), String> {
    let mods_path = get_mods_folder(&state)?;
    let folder_name = get_folder_name(&state, mod_id)?;

    if game_is_running(&state)? {
        return Err(
            "The game is currently running. Close it first — changing this while the game is \
             open could be overwritten when the game exits."
                .to_string(),
        );
    }

    keybinds::set_persist_var(&mods_path, &folder_name, &var_name, value)
}
