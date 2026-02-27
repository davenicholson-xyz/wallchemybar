use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    entries.retain(|e| e.id != wallpaper.id);
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
    let entries = load_history_entries(&app);
    let mut seen = HashSet::new();
    entries.into_iter().filter(|e| seen.insert(e.id.clone())).collect()
}

#[tauri::command]
pub fn undo_wallpaper(app: tauri::AppHandle) -> Result<(), String> {
    let entries = load_history_entries(&app);
    if entries.len() < 2 {
        return Err("No previous wallpaper to revert to".into());
    }
    let prev = &entries[1];
    let filename = prev.path.rsplit('/').next().unwrap_or("wallpaper.jpg");
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("cache dir error: {e}"))?;
    let file_path = cache_dir.join(filename);
    if !file_path.exists() {
        return Err("Previous wallpaper not in cache".into());
    }
    let settings = crate::settings::load_settings(app.clone());
    crate::setwallpaper::set(file_path.to_str().unwrap(), &settings.linux_wallpaper_cmd)?;
    // Move the previous entry to the top of history
    let mut entries = entries;
    let entry = entries.remove(1);
    entries[0] = HistoryEntry {
        applied_at: chrono::Utc::now().to_rfc3339(),
        ..entry
    };
    save_history_entries(&app, &entries)
}

#[tauri::command]
pub fn delete_history_entry(app: tauri::AppHandle, wallpaper_id: String) -> Result<(), String> {
    let mut entries = load_history_entries(&app);
    entries.retain(|e| e.id != wallpaper_id);
    save_history_entries(&app, &entries)
}

#[tauri::command]
pub fn clear_history(app: tauri::AppHandle) -> Result<(), String> {
    save_history_entries(&app, &[])
}
