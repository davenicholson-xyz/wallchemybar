use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, PhysicalPosition, WindowEvent,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub username: String,
    pub api_key: String,
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
fn load_settings(app: tauri::AppHandle) -> Settings {
    let path = settings_path(&app);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_settings, save_settings])
        .setup(|app| {
            let icon = app
                .default_window_icon()
                .cloned()
                .expect("failed to load tray icon");

            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit])?;

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("wallchemybar")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        rect,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let scale = window.scale_factor().unwrap_or(1.0);

                                let window_size =
                                    window.outer_size().unwrap_or(tauri::PhysicalSize {
                                        width: 300,
                                        height: 500,
                                    });

                                let icon_pos: PhysicalPosition<f64> =
                                    rect.position.to_physical(scale);
                                let icon_size: tauri::PhysicalSize<f64> =
                                    rect.size.to_physical(scale);

                                let x = icon_pos.x + (icon_size.width / 2.0)
                                    - (window_size.width as f64 / 2.0);
                                let y = icon_pos.y + icon_size.height;

                                let _ = window.set_position(PhysicalPosition::new(x, y));
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide main window when it loses focus
            if let Some(main_window) = app.get_webview_window("main") {
                let w = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = w.hide();
                    }
                });
            }

            // Intercept settings window close to hide instead of destroy
            if let Some(settings_window) = app.get_webview_window("settings") {
                let w = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
