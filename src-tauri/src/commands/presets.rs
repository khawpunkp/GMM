use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::models::Preset;
use crate::presets;
use crate::DbState;

fn get_mods_folder(state: &State<DbState>) -> Result<PathBuf, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let path_str: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'mods_folder_path'", [], |row| row.get(0))
        .map_err(|_| "Mods folder path is not configured. Set it in Settings first.".to_string())?;
    Ok(PathBuf::from(path_str))
}

#[tauri::command]
pub fn list_presets(state: State<DbState>) -> Result<Vec<Preset>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    presets::list_presets(&conn)
}

#[tauri::command]
pub fn create_preset(name: String, state: State<DbState>) -> Result<Preset, String> {
    let mods_path = get_mods_folder(&state)?;
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    presets::create_preset(&mut conn, &mods_path, &name)
}

#[tauri::command]
pub fn overwrite_preset(preset_id: i64, state: State<DbState>) -> Result<(), String> {
    let mods_path = get_mods_folder(&state)?;
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    presets::overwrite_preset(&mut conn, &mods_path, preset_id)
}

#[tauri::command]
pub fn delete_preset(preset_id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    presets::delete_preset(&conn, preset_id)
}

#[tauri::command]
pub fn toggle_preset_favorite(preset_id: i64, is_favorite: bool, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    presets::toggle_favorite(&conn, preset_id, is_favorite)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PresetApplyProgress {
    processed: usize,
    total: usize,
    current_mod_name: String,
}

#[tauri::command]
pub fn apply_preset(preset_id: i64, state: State<DbState>, app_handle: AppHandle) -> Result<String, String> {
    let mods_path = get_mods_folder(&state)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let result = presets::apply_preset(&conn, &mods_path, preset_id, |processed, total, current_mod_name| {
        app_handle
            .emit(
                "preset-apply-progress",
                PresetApplyProgress { processed, total, current_mod_name: current_mod_name.to_string() },
            )
            .ok();
    });

    match &result {
        Ok(summary) => {
            app_handle.emit("preset-apply-complete", summary).ok();
        }
        Err(e) => {
            app_handle.emit("preset-apply-error", e).ok();
        }
    }
    result
}
