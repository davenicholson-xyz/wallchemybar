use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub username: String,
    pub api_key: String,
    #[serde(default = "default_purity")]
    pub purity: String,
}

fn default_purity() -> String {
    "100".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: String::new(),
            api_key: String::new(),
            purity: default_purity(),
        }
    }
}

fn settings_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("failed to get config dir");
    fs::create_dir_all(&dir).ok();
    dir.join("settings.json")
}

#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Settings {
    let path = settings_path(&app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
