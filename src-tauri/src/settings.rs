use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub username: String,
    pub api_key: String,
    #[serde(default = "default_purity")]
    pub purity: String,
    #[serde(default = "default_categories")]
    pub categories: String,
    #[serde(default)]
    pub atleast: String,
    #[serde(default)]
    pub ratios: String,
    #[serde(default)]
    pub collection_cycle_collection_id: u64,
    #[serde(default = "default_collection_cycle_interval")]
    pub collection_cycle_interval_minutes: u32,
}

fn default_purity() -> String {
    "100".to_string()
}

fn default_categories() -> String {
    "111".to_string()
}

fn default_collection_cycle_interval() -> u32 {
    30
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: String::new(),
            api_key: String::new(),
            purity: default_purity(),
            categories: default_categories(),
            atleast: String::new(),
            ratios: String::new(),
            collection_cycle_collection_id: 0,
            collection_cycle_interval_minutes: default_collection_cycle_interval(),
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
