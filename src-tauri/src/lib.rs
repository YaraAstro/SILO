mod api;
mod models;
mod monitor;
mod storage;

use api::{
    delete_rule, export_logs, export_usage_data, get_app_state, get_data_usage, get_network_speed,
    get_rules, get_settings, get_usage, get_usage_90d, handshake, mark_backup_complete, save_rule,
    save_settings, start_focus_mode, stop_focus_mode, toggle_focus_mode, get_available_apps,
    get_network_history, add_rule_time,
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

#[derive(Default, Clone, Debug)]
struct RuleState {
    warned_5m: bool,
    warned_1m: bool,
    last_countdown_sec: Option<i64>,
    warned_soft_hit: bool,
    last_soft_notification_ts: Option<i64>,
}

pub struct AppState {
    storage: Storage,
    monitor: Monitor,
    network_monitor: NetworkMonitor,
    focus_mode: parking_lot::Mutex<bool>,
    rule_states: parking_lot::Mutex<std::collections::HashMap<i64, RuleState>>,
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        let storage = Storage::open_default()?;
        let rules = storage.rules()?;
        let has_rules = !rules.is_empty();

        Ok(Self {
            storage,
            monitor: Monitor::new(),
            network_monitor: NetworkMonitor::new(),
            focus_mode: parking_lot::Mutex::new(has_rules),
            rule_states: parking_lot::Mutex::new(std::collections::HashMap::new()),
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

fn close_target_app(pid: Option<u32>) {
    if let Some(pid) = pid {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let _ = Command::new("taskkill")
                .args(&["/F", "/PID", &pid.to_string()])
                .spawn();
        }
    }
}

fn minimize_active_window() {
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
}

fn show_custom_notification(
    app_handle: &tauri::AppHandle,
    rule_id: i64,
    target: &str,
    enforcement: &str,
    limit_seconds: i64,
    remaining_seconds: i64,
) {
    let settings = match app_handle.state::<AppState>().storage().settings() {
        Ok(s) => s,
        Err(_) => return,
    };
    if !settings.notifications_enabled {
        return;
    }

    let target_safe = target.replace(' ', "%20");
    let url_str = format!(
        "/notification?ruleId={}&target={}&enforcement={}&limitSeconds={}&remainingSeconds={}",
        rule_id, target_safe, enforcement, limit_seconds, remaining_seconds
    );

    if let Some(existing) = app_handle.get_webview_window("notification") {
        let _ = existing.emit("update_countdown", remaining_seconds);
        return;
    }

    let builder = tauri::WebviewWindowBuilder::new(
        app_handle,
        "notification",
        tauri::WebviewUrl::App(url_str.into()),
    )
    .title("SILO Notification")
    .inner_size(360.0, 180.0)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .transparent(true)
    .visible(false);

    if let Ok(window) = builder.build() {
        if let Ok(Some(monitor)) = app_handle.primary_monitor() {
            let size = monitor.size();
            let scale_factor = monitor.scale_factor();
            let monitor_width = (size.width as f64 / scale_factor) as i32;
            let monitor_height = (size.height as f64 / scale_factor) as i32;

            let window_width = 360;
            let window_height = 180;
            let x = monitor_width - window_width - 20;
            let y = monitor_height - window_height - 60;

            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x as f64, y as f64)));
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn check_and_enforce_rules(
    app_handle: &tauri::AppHandle,
    state: &AppState,
    active_app: &models::ActiveApp,
) -> anyhow::Result<()> {
    let rules = state.storage().rules()?;
    let mut rule_states = state.rule_states.lock();
    let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();

    for rule in rules {
        if !rule.active {
            continue;
        }

        let rule_id = match rule.id {
            Some(id) => id,
            None => continue,
        };

        let matches_active = match rule.rule_type.as_str() {
            "app" => active_app.app.to_lowercase() == rule.target.to_lowercase(),
            "site" => {
                if let Some(ref active_site) = active_app.site {
                    active_site.to_lowercase() == rule.target.to_lowercase()
                } else {
                    false
                }
            }
            _ => false,
        };

        let elapsed = match rule.rule_type.as_str() {
            "app" => {
                let db_seconds = state.storage().today_app_seconds(&rule.target)?;
                let current_extra = if matches_active {
                    active_app.elapsed_seconds
                } else {
                    0
                };
                db_seconds + current_extra
            }
            "site" => {
                state.storage().today_site_seconds(&rule.target)?
            }
            _ => 0,
        };

        let limit = if rule.extra_limit_date.as_ref() == Some(&today_str) {
            rule.limit_seconds + rule.extra_limit_seconds
        } else {
            rule.limit_seconds
        };

        let remaining = limit - elapsed;
        let rule_state = rule_states.entry(rule_id).or_insert_with(RuleState::default);

        if remaining > 300 {
            rule_state.warned_5m = false;
        }
        if remaining > 60 {
            rule_state.warned_1m = false;
            rule_state.last_countdown_sec = None;
        }
        if remaining > 0 {
            rule_state.warned_soft_hit = false;
        }

        match rule.enforcement.as_str() {
            "hard" => {
                if matches_active && remaining > 0 && remaining <= 60 {
                    let _ = app_handle.emit(
                        "rule_countdown",
                        serde_json::json!({
                            "ruleId": rule_id,
                            "target": rule.target,
                            "remainingSeconds": remaining,
                            "enforcement": "hard"
                        }),
                    );

                    if remaining == 60 || remaining == 30 || remaining == 10 || remaining <= 5 {
                        if rule_state.last_countdown_sec != Some(remaining) {
                            rule_state.last_countdown_sec = Some(remaining);
                            show_custom_notification(app_handle, rule_id, &rule.target, "hard", limit, remaining);
                        }
                    }
                }

                if matches_active && remaining <= 0 {
                    close_target_app(active_app.pid);

                    if !rule_state.warned_soft_hit {
                        rule_state.warned_soft_hit = true;
                        
                        show_custom_notification(app_handle, rule_id, &rule.target, "hard", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_violated",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "ruleType": rule.rule_type,
                                "enforcement": "hard",
                                "limitSeconds": limit,
                                "remainingSeconds": remaining,
                            }),
                        );
                    }
                }
            }
            "warn" => {
                if matches_active && remaining > 0 {
                    if remaining <= 300 && remaining > 60 && !rule_state.warned_5m {
                        rule_state.warned_5m = true;

                        show_custom_notification(app_handle, rule_id, &rule.target, "warn", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_warning",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "ruleType": rule.rule_type,
                                "enforcement": "warn",
                                "limitSeconds": limit,
                                "remainingSeconds": remaining,
                            }),
                        );
                    }

                    if remaining <= 60 && !rule_state.warned_1m {
                        rule_state.warned_1m = true;

                        show_custom_notification(app_handle, rule_id, &rule.target, "warn", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_warning",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "ruleType": rule.rule_type,
                                "enforcement": "warn",
                                "limitSeconds": limit,
                                "remainingSeconds": remaining,
                            }),
                        );
                    }
                    
                    if remaining <= 60 {
                        show_custom_notification(app_handle, rule_id, &rule.target, "warn", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_countdown",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "remainingSeconds": remaining,
                                "enforcement": "warn"
                            }),
                        );
                    }
                }

                if matches_active && remaining <= 0 {
                    close_target_app(active_app.pid);

                    if !rule_state.warned_soft_hit {
                        rule_state.warned_soft_hit = true;

                        show_custom_notification(app_handle, rule_id, &rule.target, "warn", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_violated",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "ruleType": rule.rule_type,
                                "enforcement": "warn",
                                "limitSeconds": limit,
                                "remainingSeconds": remaining,
                            }),
                        );
                    }
                }
            }
            "soft" => {
                if matches_active && remaining <= 0 {
                    minimize_active_window();

                    let now_ts = chrono::Utc::now().timestamp();
                    let should_notify = match rule_state.last_soft_notification_ts {
                        Some(ts) => now_ts - ts >= 10,
                        None => true,
                    };

                    if should_notify {
                        rule_state.last_soft_notification_ts = Some(now_ts);

                        show_custom_notification(app_handle, rule_id, &rule.target, "soft", limit, remaining);

                        let _ = app_handle.emit(
                            "rule_violated",
                            serde_json::json!({
                                "ruleId": rule_id,
                                "target": rule.target,
                                "ruleType": rule.rule_type,
                                "enforcement": "soft",
                                "limitSeconds": limit,
                                "remainingSeconds": remaining,
                            }),
                        );
                    }
                }
            }
            _ => {}
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
            get_network_history,
            add_rule_time
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
