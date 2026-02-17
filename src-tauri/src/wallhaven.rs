use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

use crate::settings::load_settings;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
    pub path: String,
    pub thumbs: Thumbs,
    pub resolution: String,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thumbs {
    pub large: String,
    pub original: String,
    pub small: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: u64,
    pub label: String,
    pub views: u64,
    pub public: u8,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Vec<Wallpaper>,
}

#[derive(Debug, Deserialize)]
struct WallpaperInfoResponse {
    data: WallpaperInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WallpaperInfo {
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
struct CollectionsResponse {
    data: Vec<Collection>,
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("wallchemybar/0.1.0")
        .build()
        .map_err(|e| format!("client error: {e}"))
}

#[tauri::command]
pub async fn fetch_search(
    app: tauri::AppHandle,
    sorting: String,
    page: Option<u32>,
    query: Option<String>,
) -> Result<Vec<Wallpaper>, String> {
    let settings = load_settings(app);
    let client = build_client()?;
    let page_str = page.unwrap_or(1).to_string();
    let purity = settings.purity.clone();
    let categories = settings.categories.clone();
    let atleast = settings.atleast.clone();
    let query_str = query.unwrap_or_default();
    let mut req = client.get("https://wallhaven.cc/api/v1/search").query(&[
        ("sorting", sorting.as_str()),
        ("purity", purity.as_str()),
        ("categories", categories.as_str()),
        ("page", page_str.as_str()),
    ]);
    if !query_str.is_empty() {
        req = req.query(&[("q", query_str.as_str())]);
    }
    if !atleast.is_empty() {
        req = req.query(&[("atleast", atleast.as_str())]);
    }

    let api_key = settings.api_key.trim().to_string();
    if !api_key.is_empty() {
        req = req.header("X-API-Key", &api_key);
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    // If unauthorized (bad API key), retry without it
    let text = if response.status() == 401 && !api_key.is_empty() {
        let mut retry = client.get("https://wallhaven.cc/api/v1/search").query(&[
            ("sorting", sorting.as_str()),
            ("purity", "100"),
            ("categories", categories.as_str()),
            ("page", page_str.as_str()),
        ]);
        if !query_str.is_empty() {
            retry = retry.query(&[("q", query_str.as_str())]);
        }
        if !atleast.is_empty() {
            retry = retry.query(&[("atleast", atleast.as_str())]);
        }
        retry
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?
            .text()
            .await
            .map_err(|e| format!("reading body failed: {e}"))?
    } else {
        response
            .text()
            .await
            .map_err(|e| format!("reading body failed: {e}"))?
    };

    let resp: SearchResponse =
        serde_json::from_str(&text).map_err(|e| format!("parse failed: {e}"))?;

    Ok(resp.data)
}

#[tauri::command]
pub async fn fetch_collections(app: tauri::AppHandle) -> Result<Vec<Collection>, String> {
    let settings = load_settings(app);
    let username = settings.username.trim().to_string();
    if username.is_empty() {
        return Err("Username not configured. Set it in Settings.".into());
    }

    let client = build_client()?;
    let mut req = client.get(format!(
        "https://wallhaven.cc/api/v1/collections/{username}"
    ));

    let api_key = settings.api_key.trim().to_string();
    if !api_key.is_empty() {
        req = req.header("X-API-Key", &api_key);
    }

    let text = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("reading body failed: {e}"))?;

    let resp: CollectionsResponse =
        serde_json::from_str(&text).map_err(|e| format!("parse failed: {e}"))?;

    Ok(resp.data)
}

#[tauri::command]
pub async fn set_wallpaper(app: tauri::AppHandle, wallpaper: Wallpaper) -> Result<(), String> {
    // Extract filename from URL
    let filename = wallpaper.path.rsplit('/').next().unwrap_or("wallpaper.jpg");

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("cache dir error: {e}"))?;
    fs::create_dir_all(&cache_dir).ok();
    let file_path = cache_dir.join(filename);

    if !file_path.exists() {
        let client = build_client()?;

        let settings = load_settings(app.clone());
        let api_key = settings.api_key.trim().to_string();

        let mut req = client.get(&wallpaper.path);
        if !api_key.is_empty() {
            req = req.header("X-API-Key", &api_key);
        }

        let bytes = req
            .send()
            .await
            .map_err(|e| format!("download failed: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("reading image failed: {e}"))?;

        fs::write(&file_path, &bytes).map_err(|e| format!("write failed: {e}"))?;
    }

    crate::setwallpaper::set(file_path.to_str().unwrap())?;

    crate::history::add_to_history(&app, &wallpaper)?;

    Ok(())
}

#[tauri::command]
pub async fn fetch_collection_wallpapers(
    app: tauri::AppHandle,
    collection_id: u64,
    page: Option<u32>,
) -> Result<Vec<Wallpaper>, String> {
    let settings = load_settings(app);
    let username = settings.username.trim().to_string();
    if username.is_empty() {
        return Err("Username not configured. Set it in Settings.".into());
    }

    let client = build_client()?;
    let mut req = client.get(format!(
        "https://wallhaven.cc/api/v1/collections/{username}/{collection_id}"
    ));

    let api_key = settings.api_key.trim().to_string();
    if !api_key.is_empty() {
        req = req.header("X-API-Key", &api_key);
    }

    if let Some(p) = page {
        req = req.query(&[("page", p.to_string())]);
    }

    let text = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("reading body failed: {e}"))?;

    let resp: SearchResponse =
        serde_json::from_str(&text).map_err(|e| format!("parse failed: {e}"))?;

    Ok(resp.data)
}

#[tauri::command]
pub async fn fetch_wallpaper_tags(
    app: tauri::AppHandle,
    wallpaper_id: String,
) -> Result<Vec<Tag>, String> {
    let settings = load_settings(app);
    let client = build_client()?;

    let mut req = client.get(format!(
        "https://wallhaven.cc/api/v1/w/{wallpaper_id}"
    ));

    let api_key = settings.api_key.trim().to_string();
    if !api_key.is_empty() {
        req = req.header("X-API-Key", &api_key);
    }

    let text = req
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("reading body failed: {e}"))?;

    let resp: WallpaperInfoResponse =
        serde_json::from_str(&text).map_err(|e| format!("parse failed: {e}"))?;

    Ok(resp.data.tags)
}
