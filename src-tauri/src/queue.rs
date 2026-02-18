use std::fs;
use tauri::Manager;

use crate::wallhaven::Wallpaper;

fn queue_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("failed to get config dir");
    fs::create_dir_all(&dir).ok();
    dir.join("queue.json")
}

fn load_queue_entries(app: &tauri::AppHandle) -> Vec<Wallpaper> {
    let path = queue_path(app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_queue_entries(app: &tauri::AppHandle, entries: &[Wallpaper]) -> Result<(), String> {
    let path = queue_path(app);
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_queue(app: tauri::AppHandle) -> Vec<Wallpaper> {
    load_queue_entries(&app)
}

#[tauri::command]
pub fn add_to_queue(app: tauri::AppHandle, wallpaper: Wallpaper) -> Result<(), String> {
    let mut entries = load_queue_entries(&app);
    if entries.iter().any(|e| e.id == wallpaper.id) {
        return Ok(());
    }
    entries.push(wallpaper);
    save_queue_entries(&app, &entries)
}

#[tauri::command]
pub fn remove_from_queue(app: tauri::AppHandle, wallpaper_id: String) -> Result<(), String> {
    let mut entries = load_queue_entries(&app);
    entries.retain(|e| e.id != wallpaper_id);
    save_queue_entries(&app, &entries)
}

#[tauri::command]
pub fn reorder_queue(app: tauri::AppHandle, wallpapers: Vec<Wallpaper>) -> Result<(), String> {
    save_queue_entries(&app, &wallpapers)
}

#[tauri::command]
pub fn clear_queue(app: tauri::AppHandle) -> Result<(), String> {
    save_queue_entries(&app, &[])
}
