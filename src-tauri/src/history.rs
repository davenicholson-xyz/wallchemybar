use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

use crate::wallhaven::{Thumbs, Wallpaper};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub path: String,
    pub thumbs: Thumbs,
    pub resolution: String,
    pub applied_at: String,
}

fn history_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("failed to get config dir");
    fs::create_dir_all(&dir).ok();
    dir.join("history.json")
}

fn load_history_entries(app: &tauri::AppHandle) -> Vec<HistoryEntry> {
    let path = history_path(app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_history_entries(app: &tauri::AppHandle, entries: &[HistoryEntry]) -> Result<(), String> {
    let path = history_path(app);
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn add_to_history(app: &tauri::AppHandle, wallpaper: &Wallpaper) -> Result<(), String> {
    let mut entries = load_history_entries(app);
    let entry = HistoryEntry {
        id: wallpaper.id.clone(),
        url: wallpaper.url.clone(),
        path: wallpaper.path.clone(),
        thumbs: wallpaper.thumbs.clone(),
        resolution: wallpaper.resolution.clone(),
        applied_at: chrono::Utc::now().to_rfc3339(),
    };
    entries.insert(0, entry);
    save_history_entries(app, &entries)
}

#[tauri::command]
pub fn get_history(app: tauri::AppHandle) -> Vec<HistoryEntry> {
    load_history_entries(&app)
}

#[tauri::command]
pub fn clear_history(app: tauri::AppHandle) -> Result<(), String> {
    save_history_entries(&app, &[])
}
