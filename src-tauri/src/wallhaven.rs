use log::{debug, error, info, warn};
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
    let page_num = page.unwrap_or(1);
    info!("fetch_search: sorting={}, page={}, query={:?}", sorting, page_num, query);

    let settings = load_settings(app);
    let client = build_client()?;
    let page_str = page_num.to_string();
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
        .map_err(|e| {
            error!("fetch_search: request failed: {e}");
            format!("request failed: {e}")
        })?;

    debug!("fetch_search: response status={}", response.status());

    // If unauthorized (bad API key), retry without it
    let text = if response.status() == 401 && !api_key.is_empty() {
        warn!("fetch_search: got 401, retrying without API key");
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
            .map_err(|e| {
                error!("fetch_search: retry request failed: {e}");
                format!("request failed: {e}")
            })?
            .text()
            .await
            .map_err(|e| {
                error!("fetch_search: retry reading body failed: {e}");
                format!("reading body failed: {e}")
            })?
    } else if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("fetch_search: API returned status={}, body={}", status, &body[..body.len().min(500)]);
        return Err(format!("API error: status {status}"));
    } else {
        response
            .text()
            .await
            .map_err(|e| {
                error!("fetch_search: reading body failed: {e}");
                format!("reading body failed: {e}")
            })?
    };

    let resp: SearchResponse =
        serde_json::from_str(&text).map_err(|e| {
            error!("fetch_search: parse failed: {e}, body={}", &text[..text.len().min(500)]);
            format!("parse failed: {e}")
        })?;

    info!("fetch_search: returned {} wallpapers for page {}", resp.data.len(), page_num);
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
    info!("set_wallpaper: id={}, path={}", wallpaper.id, wallpaper.path);

    let filename = wallpaper.path.rsplit('/').next().unwrap_or("wallpaper.jpg");

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("cache dir error: {e}"))?;
    fs::create_dir_all(&cache_dir).ok();
    let file_path = cache_dir.join(filename);

    if file_path.exists() {
        debug!("set_wallpaper: using cached file {:?}", file_path);
    } else {
        info!("set_wallpaper: downloading to {:?}", file_path);
        let client = build_client()?;

        let settings = load_settings(app.clone());
        let api_key = settings.api_key.trim().to_string();

        let mut req = client.get(&wallpaper.path);
        if !api_key.is_empty() {
            req = req.header("X-API-Key", &api_key);
        }

        let response = req
            .send()
            .await
            .map_err(|e| {
                error!("set_wallpaper: download failed: {e}");
                format!("download failed: {e}")
            })?;

        debug!("set_wallpaper: download status={}", response.status());

        let bytes = response
            .bytes()
            .await
            .map_err(|e| {
                error!("set_wallpaper: reading image failed: {e}");
                format!("reading image failed: {e}")
            })?;

        info!("set_wallpaper: downloaded {} bytes", bytes.len());
        fs::write(&file_path, &bytes).map_err(|e| {
            error!("set_wallpaper: write failed: {e}");
            format!("write failed: {e}")
        })?;
    }

    crate::setwallpaper::set(file_path.to_str().unwrap())?;
    crate::history::add_to_history(&app, &wallpaper)?;

    info!("set_wallpaper: applied successfully");
    Ok(())
}

#[tauri::command]
pub async fn fetch_collection_wallpapers(
    app: tauri::AppHandle,
    collection_id: u64,
    page: Option<u32>,
) -> Result<Vec<Wallpaper>, String> {
    info!("fetch_collection_wallpapers: collection_id={}, page={:?}", collection_id, page);

    let settings = load_settings(app);
    let username = settings.username.trim().to_string();
    if username.is_empty() {
        warn!("fetch_collection_wallpapers: username not configured");
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

    let response = req
        .send()
        .await
        .map_err(|e| {
            error!("fetch_collection_wallpapers: request failed: {e}");
            format!("request failed: {e}")
        })?;

    debug!("fetch_collection_wallpapers: response status={}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("fetch_collection_wallpapers: API returned status={}, body={}", status, &body[..body.len().min(500)]);
        return Err(format!("API error: status {status}"));
    }

    let text = response
        .text()
        .await
        .map_err(|e| {
            error!("fetch_collection_wallpapers: reading body failed: {e}");
            format!("reading body failed: {e}")
        })?;

    let resp: SearchResponse =
        serde_json::from_str(&text).map_err(|e| {
            error!("fetch_collection_wallpapers: parse failed: {e}, body={}", &text[..text.len().min(500)]);
            format!("parse failed: {e}")
        })?;

    info!("fetch_collection_wallpapers: returned {} wallpapers", resp.data.len());
    Ok(resp.data)
}

#[tauri::command]
pub async fn fetch_wallpaper_tags(
    app: tauri::AppHandle,
    wallpaper_id: String,
) -> Result<Vec<Tag>, String> {
    debug!("fetch_wallpaper_tags: wallpaper_id={}", wallpaper_id);

    let settings = load_settings(app);
    let client = build_client()?;

    let mut req = client.get(format!(
        "https://wallhaven.cc/api/v1/w/{wallpaper_id}"
    ));

    let api_key = settings.api_key.trim().to_string();
    if !api_key.is_empty() {
        req = req.header("X-API-Key", &api_key);
    }

    let response = req
        .send()
        .await
        .map_err(|e| {
            error!("fetch_wallpaper_tags: request failed: {e}");
            format!("request failed: {e}")
        })?;

    if !response.status().is_success() {
        let status = response.status();
        warn!("fetch_wallpaper_tags: API returned status={}", status);
        return Err(format!("API error: status {status}"));
    }

    let text = response
        .text()
        .await
        .map_err(|e| {
            error!("fetch_wallpaper_tags: reading body failed: {e}");
            format!("reading body failed: {e}")
        })?;

    let resp: WallpaperInfoResponse =
        serde_json::from_str(&text).map_err(|e| {
            error!("fetch_wallpaper_tags: parse failed: {e}");
            format!("parse failed: {e}")
        })?;

    debug!("fetch_wallpaper_tags: returned {} tags", resp.data.tags.len());
    Ok(resp.data.tags)
}
