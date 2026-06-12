mod api;
mod models;
mod monitor;
mod storage;

use api::{
    delete_rule, export_logs, export_usage_data, get_app_state, get_data_usage, get_network_speed,
    get_rules, get_settings, get_usage, get_usage_90d, handshake, mark_backup_complete, save_rule,
    save_settings, start_focus_mode, stop_focus_mode, toggle_focus_mode, get_available_apps,
    get_network_history,
};
use models::AppReadyEvent;
use monitor::{Monitor, NetworkMonitor};
use std::time::Duration;
use storage::Storage;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent, Listener,
};

pub struct AppState {
    storage: Storage,
    monitor: Monitor,
    network_monitor: NetworkMonitor,
    focus_mode: parking_lot::Mutex<bool>,
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            storage: Storage::open_default()?,
            monitor: Monitor::new(),
            network_monitor: NetworkMonitor::new(),
            focus_mode: parking_lot::Mutex::new(false),
        })
    }

    fn storage(&self) -> &Storage {
        &self.storage
    }

    fn focus_mode_enabled(&self) -> bool {
        *self.focus_mode.lock()
    }

    fn active_app(&self) -> models::ActiveApp {
        self.sample_active_app_and_persist()
    }

    fn network_speed(&self) -> models::NetworkSpeed {
        self.network_monitor.sample_speed()
    }

    fn sample_active_app_and_persist(&self) -> models::ActiveApp {
        let (active_app, completed_session) = self.monitor.sample_active_app_with_session();
        if let Some(session) = completed_session {
            if let Err(error) = self.storage.save_completed_session(&session) {
                tracing::warn!("failed to save completed session: {error}");
            }
        }
        active_app
    }

    fn set_focus_mode(&self, enabled: bool) {
        *self.focus_mode.lock() = enabled;
    }
}

fn spawn_monitoring(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let state = app_handle.state::<AppState>();
            let active_app = state.sample_active_app_and_persist();
            
            if let Some(ref site) = active_app.site {
                let _ = state.storage().track_site_time(site, 1);
            }

            // Track live network bytes attribution for daily usage
            let speed = state.network_speed();
            if speed.upload_bps > 0 || speed.download_bps > 0 {
                let _ = state.storage().track_app_data_usage(&active_app.app, speed.upload_bps, speed.download_bps);
                if let Some(ref site) = active_app.site {
                    let _ = state.storage().track_site_data_usage(site, speed.upload_bps, speed.download_bps);
                }
            }

            let today_seconds = state.storage().today_seconds().unwrap_or_default();

            let _ = app_handle.emit("update_active_app", &active_app);
            let _ = app_handle.emit(
                "usage_update",
                serde_json::json!({
                    "app": active_app.app,
                    "todaySeconds": today_seconds
                }),
            );

            if state.focus_mode_enabled() {
                let _ = check_and_enforce_rules(&app_handle, &state, &active_app);
            }
        }
    });
}

fn check_and_enforce_rules(
    app_handle: &tauri::AppHandle,
    state: &AppState,
    active_app: &models::ActiveApp,
) -> anyhow::Result<()> {
    let rules = state.storage().rules()?;
    for rule in rules {
        if !rule.active {
            continue;
        }

        let is_violated = match rule.rule_type.as_str() {
            "app" => {
                if active_app.app.to_lowercase() == rule.target.to_lowercase() {
                    let today_seconds = state.storage().today_app_seconds(&active_app.app)?;
                    today_seconds > rule.limit_seconds
                } else {
                    false
                }
            }
            "site" => {
                if let Some(ref active_site) = active_app.site {
                    if active_site.to_lowercase() == rule.target.to_lowercase() {
                        let today_seconds = state.storage().today_site_seconds(active_site)?;
                        today_seconds > rule.limit_seconds
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_violated {
            let _ = app_handle.emit(
                "rule_violated",
                serde_json::json!({
                    "target": rule.target,
                    "ruleType": rule.rule_type,
                    "enforcement": rule.enforcement,
                    "limitSeconds": rule.limit_seconds,
                }),
            );

            if rule.enforcement == "hard" {
                #[cfg(target_os = "windows")]
                {
                    unsafe {
                        use windows::Win32::UI::WindowsAndMessaging::{
                            GetForegroundWindow, ShowWindow, SW_MINIMIZE,
                        };
                        let hwnd = GetForegroundWindow();
                        if !hwnd.0.is_null() {
                            let _ = ShowWindow(hwnd, SW_MINIMIZE);
                        }
                    }
                }

                let settings = state.storage().settings()?;
                if settings.notifications_enabled {
                    use tauri_plugin_notification::NotificationExt;
                    let _ = app_handle
                        .notification()
                        .builder()
                        .title("Silo Focus Block")
                        .body(format!(
                            "Time limit exceeded for {}! Window minimized.",
                            rule.target
                        ))
                        .show();
                }
            }
            break;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_target(false).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let state = AppState::new()?;
            app.manage(state);
            spawn_monitoring(app.handle().clone());

            let show_i = MenuItemBuilder::with_id("show", "Show SILO").build(app)?;
            let add_rule_i = MenuItemBuilder::with_id("add_rule", "Add Rule").build(app)?;
            let app_state = app.state::<AppState>();
            let toggle_focus_text = if app_state.focus_mode_enabled() {
                "Stop Focus Mode"
            } else {
                "Start Focus Mode"
            };
            let toggle_focus_i = MenuItemBuilder::with_id("toggle_focus", toggle_focus_text).build(app)?;
            let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let separator1 = PredefinedMenuItem::separator(app)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[
                    &show_i,
                    &add_rule_i,
                    &separator1,
                    &toggle_focus_i,
                    &separator2,
                    &quit_i,
                ])
                .build()?;

            let toggle_focus_clone = toggle_focus_i.clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "add_rule" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = app.emit("navigate", "rules");
                            }
                        }
                        "toggle_focus" => {
                            let app_state = app.state::<AppState>();
                            let enabled = !app_state.focus_mode_enabled();
                            app_state.set_focus_mode(enabled);
                            let _ = app.emit("focus_mode_changed", serde_json::json!({ "enabled": enabled }));
                            
                            let new_text = if enabled {
                                "Stop Focus Mode"
                            } else {
                                "Start Focus Mode"
                            };
                            let _ = toggle_focus_clone.set_text(new_text);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(true) = window.is_visible() {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let toggle_focus_listener = toggle_focus_i.clone();
            app.listen_any("focus_mode_changed", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
                        let new_text = if enabled {
                            "Stop Focus Mode"
                        } else {
                            "Start Focus Mode"
                        };
                        let _ = toggle_focus_listener.set_text(new_text);
                    }
                }
            });

            app.emit(
                "app_ready",
                AppReadyEvent {
                    status: "ready".to_string(),
                    message: "SILO backend initialized".to_string(),
                },
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            handshake,
            get_app_state,
            start_focus_mode,
            stop_focus_mode,
            toggle_focus_mode,
            get_rules,
            save_rule,
            delete_rule,
            get_usage,
            get_usage_90d,
            get_network_speed,
            get_data_usage,
            export_logs,
            export_usage_data,
            get_settings,
            save_settings,
            mark_backup_complete,
            get_available_apps,
            get_network_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
