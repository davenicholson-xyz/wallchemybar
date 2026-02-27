mod history;
mod queue;
mod settings;
mod setwallpaper;
mod wallhaven;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, PhysicalPosition, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn hide_main(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn open_expanded(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
    if let Some(w) = app.get_webview_window("expanded") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn close_expanded(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("expanded") {
        let _ = w.hide();
    }
}

fn handle_cli_args(app: &tauri::AppHandle, argv: &[String]) {
    let action = argv.iter().skip(1)
        .find_map(|a| match a.trim_start_matches('-') {
            "show" | "s"     => Some("show"),
            "toggle" | "t"   => Some("toggle"),
            "expanded" | "e" => Some("expanded"),
            "hide" | "h"     => Some("hide"),
            _                => None,
        })
        .unwrap_or("toggle"); // no args → toggle

    log::info!("cli action: {}", action);
    match action {
        "expanded" => {
            if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
            if let Some(w) = app.get_webview_window("expanded") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        "hide" => {
            if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
        }
        "toggle" => {
            if let Some(w) = app.get_webview_window("main") {
                if w.is_visible().unwrap_or(false) {
                    let _ = w.hide();
                } else {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }
        _ => { // "show"
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("wallchemybar starting up");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            log::info!("second instance argv: {:?}", argv);
            handle_cli_args(app, &argv);
        }))
        .invoke_handler(tauri::generate_handler![
            settings::load_settings,
            settings::save_settings,
            wallhaven::fetch_search,
            wallhaven::fetch_collections,
            wallhaven::fetch_collection_wallpapers,
            wallhaven::set_wallpaper,
            wallhaven::fetch_wallpaper_tags,
            wallhaven::validate_api_key,
            history::get_history,
            history::clear_history,
            history::delete_history_entry,
            history::undo_wallpaper,
            queue::get_queue,
            queue::add_to_queue,
            queue::remove_from_queue,
            queue::reorder_queue,
            queue::clear_queue,
            hide_main,
            quit_app,
            open_expanded,
            close_expanded
        ])
        .setup(|app| {
            // Hide the app from the macOS Dock — it lives only in the menu bar
            #[cfg(target_os = "macos")]
            unsafe {
                let ns_app: *mut objc::runtime::Object =
                    msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, setActivationPolicy: 1i64]; // NSApplicationActivationPolicyAccessory
            }

            // If launched with CLI args (e.g. from i3 keybinding when app wasn't running),
            // act on them now that windows exist.
            let launch_args: Vec<String> = std::env::args().collect();
            if launch_args.len() > 1 {
                handle_cli_args(app.handle(), &launch_args);
            }

            // Cmd+Shift+W — toggle window from anywhere
            let toggle = Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyW);
            app.handle().global_shortcut().on_shortcut(toggle, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })?;

            // Cmd+Shift+E — open/focus expanded window from anywhere
            let expand = Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyE);
            app.handle().global_shortcut().on_shortcut(expand, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    if let Some(expanded) = app.get_webview_window("expanded") {
                        if expanded.is_visible().unwrap_or(false) {
                            let _ = expanded.hide();
                        } else {
                            if let Some(main) = app.get_webview_window("main") {
                                let _ = main.hide();
                            }
                            let _ = expanded.show();
                            let _ = expanded.set_focus();
                        }
                    }
                }
            })?;

            let icon = app
                .default_window_icon()
                .cloned()
                .expect("failed to load tray icon");

            // On Ubuntu/GNOME with AppIndicator, the tray icon requires a menu to appear at all,
            // and left click always shows the menu (show_menu_on_left_click(false) is ignored).
            // On macOS/Windows, left click fires the click event directly — no menu needed.
            // Linux uses "Open" (always shows) rather than "Show / Hide" (toggle) to be clear.
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            #[cfg(target_os = "linux")]
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            #[cfg(not(target_os = "linux"))]
            let show = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
            #[cfg(target_os = "linux")]
            let menu = Menu::with_items(app, &[&open, &quit])?;
            #[cfg(not(target_os = "linux"))]
            let menu = Menu::with_items(app, &[&show, &quit])?;

            // Guard against the Windows focus-loss race: clicking the tray icon
            // moves OS focus to the notification area, firing FocusLost on our
            // window *before* the tray click event arrives. Without this flag the
            // FocusLost handler hides the window and the tray handler immediately
            // shows it again (or vice-versa), so the window never stays visible.
            let just_shown = Arc::new(AtomicBool::new(false));
            let just_shown_tray = just_shown.clone();
            let just_shown_focus = just_shown.clone();

            let tray_builder = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("wallchemybar")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" | "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            #[cfg(target_os = "linux")]
                            {
                                // On Linux/AppIndicator, always show (left click can't hide via event)
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            #[cfg(not(target_os = "linux"))]
                            {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                });

            tray_builder
                .on_tray_icon_event(move |tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        rect,
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                just_shown_tray.store(false, Ordering::Relaxed);
                                let _ = window.hide();
                            } else {
                                let scale = window.scale_factor().unwrap_or(1.0);

                                let window_size =
                                    window.outer_size().unwrap_or(tauri::PhysicalSize {
                                        width: 500,
                                        height: 500,
                                    });

                                let icon_pos: PhysicalPosition<f64> =
                                    rect.position.to_physical(scale);
                                let icon_size: tauri::PhysicalSize<f64> =
                                    rect.size.to_physical(scale);

                                // Detect screen bounds so we can position the window
                                // above the tray icon on Windows (bottom taskbar) or
                                // below it on macOS (top menu bar).
                                let (screen_w, screen_h) = window
                                    .current_monitor()
                                    .ok()
                                    .flatten()
                                    .map(|m| {
                                        (m.size().width as f64, m.size().height as f64)
                                    })
                                    .unwrap_or((1920.0, 1080.0));

                                let icon_center_y = icon_pos.y + icon_size.height / 2.0;
                                let y = if icon_center_y > screen_h / 2.0 {
                                    // Bottom half of screen — place window above icon
                                    icon_pos.y - window_size.height as f64
                                } else {
                                    // Top half of screen — place window below icon
                                    icon_pos.y + icon_size.height
                                };

                                // Clamp x so the window doesn't overflow the right edge
                                let x = (icon_pos.x + icon_size.width / 2.0
                                    - window_size.width as f64 / 2.0)
                                    .max(0.0)
                                    .min(screen_w - window_size.width as f64);

                                just_shown_tray.store(true, Ordering::Relaxed);
                                let _ = window.set_position(PhysicalPosition::new(x, y));
                                let _ = window.show();
                                let _ = window.set_focus();

                                // Clear the guard after a short grace period
                                let flag = just_shown_tray.clone();
                                std::thread::spawn(move || {
                                    std::thread::sleep(std::time::Duration::from_millis(300));
                                    flag.store(false, Ordering::Relaxed);
                                });
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide main window when it loses focus, but not during the grace
            // period right after we show it (works around Windows focus-race).
            if let Some(main_window) = app.get_webview_window("main") {
                let w = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        if !just_shown_focus.load(Ordering::Relaxed) {
                            let _ = w.hide();
                        }
                    }
                });
            }

            // Intercept the close button on the expanded window so it hides
            // rather than being destroyed — otherwise it can't be reopened.
            if let Some(expanded_window) = app.get_webview_window("expanded") {
                let w = expanded_window.clone();
                expanded_window.on_window_event(move |event| {
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
