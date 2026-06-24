mod api;
mod models;
mod monitor;
mod storage;

use api::{
    add_rule_time, delete_rule, export_logs, export_usage_data, get_app_state, get_available_apps,
    get_data_usage, get_network_history, get_network_speed, get_rule_stats, get_rules,
    get_settings, get_usage, get_usage_90d, get_usage_range, handshake, mark_backup_complete,
    save_rule, save_settings, start_focus_mode, stop_focus_mode, toggle_focus_mode,
};
use models::AppReadyEvent;
use monitor::{Monitor, NetworkMonitor};
use std::time::Duration;
use storage::Storage;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager, WindowEvent,
};

#[derive(Default, Clone, Debug)]
struct RuleState {
    warned_5m: bool,
    warned_1m: bool,
    last_countdown_sec: Option<i64>,
    warned_soft_hit: bool,
    last_soft_notification_ts: Option<i64>,
    last_blocked_ts: Option<i64>,
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
                let _ = state.storage().track_app_data_usage(
                    &active_app.app,
                    speed.upload_bps,
                    speed.download_bps,
                );
                if let Some(ref site) = active_app.site {
                    let _ = state.storage().track_site_data_usage(
                        site,
                        speed.upload_bps,
                        speed.download_bps,
                    );
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
                .args(["/F", "/PID", &pid.to_string()])
                .spawn();
        }
    }
}

fn minimize_active_window() {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, ShowWindow, SW_FORCEMINIMIZE,
            };
            let hwnd = GetForegroundWindow();
            if !hwnd.0.is_null() {
                let _ = ShowWindow(hwnd, SW_FORCEMINIMIZE);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn register_custom_protocol() -> Result<(), Box<dyn std::error::Error>> {
    use std::env::current_exe;
    use std::process::Command;

    let exe_path = current_exe()?;
    let exe_str = exe_path.to_string_lossy().to_string();

    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Classes\\silo",
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            "URL:silo Protocol",
            "/f",
        ])
        .status();

    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Classes\\silo",
            "/v",
            "URL Protocol",
            "/t",
            "REG_SZ",
            "/d",
            "",
            "/f",
        ])
        .status();

    let cmd_val = format!("\"{}\" \"%1\"", exe_str);
    let _ = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Classes\\silo\\shell\\open\\command",
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &cmd_val,
            "/f",
        ])
        .status();

    Ok(())
}

pub fn extend_rule_limit(app: &tauri::AppHandle, id: i64, seconds: i64) -> Result<(), String> {
    let state = app.state::<AppState>();
    let storage = state.storage();
    let rules = storage.rules().map_err(|e| e.to_string())?;
    if let Some(mut rule) = rules.into_iter().find(|r| r.id == Some(id)) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if rule.extra_limit_date.as_ref() == Some(&today) {
            rule.extra_limit_seconds += seconds;
        } else {
            rule.extra_limit_date = Some(today);
            rule.extra_limit_seconds = seconds;
        }
        let saved = storage.save_rule(rule).map_err(|e| e.to_string())?;
        let _ = app.emit("rules_changed", &saved);

        let _ = storage.increment_rule_bypassed(id);

        let mut rule_states = state.rule_states.lock();
        if let Some(state) = rule_states.get_mut(&id) {
            state.warned_5m = false;
            state.warned_1m = false;
            state.last_countdown_sec = None;
            state.warned_soft_hit = false;
        }
        Ok(())
    } else {
        Err("Rule not found".to_string())
    }
}

#[cfg(target_os = "windows")]
fn show_native_toast(
    rule_id: i64,
    _target: &str,
    enforcement: &str,
    _limit_seconds: i64,
    remaining_seconds: i64,
    title: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use windows::core::HSTRING;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

    let actions_xml = if enforcement == "warn" {
        format!(
            r#"<actions>
              <action content="+15 Min" arguments="silo://add-time/{}/{}" activationType="protocol"/>
              <action content="+30 Min" arguments="silo://add-time/{}/{}" activationType="protocol"/>
              <action content="+1 Hour" arguments="silo://add-time/{}/{}" activationType="protocol"/>
            </actions>"#,
            rule_id, 900, rule_id, 1800, rule_id, 3600
        )
    } else {
        "".to_string()
    };

    let audio_xml = if remaining_seconds > 0 && remaining_seconds < 60 {
        "<audio silent=\"true\"/>"
    } else {
        ""
    };

    let xml_str = format!(
        r#"<toast>
          <visual>
            <binding template="ToastGeneric">
              <text>{}</text>
              <text>{}</text>
            </binding>
          </visual>
          {}
          {}
        </toast>"#,
        title, body, actions_xml, audio_xml
    );

    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(&xml_str))?;

    let toast = ToastNotification::CreateToastNotification(&doc)?;
    let tag = HSTRING::from(format!("rule-{}", rule_id));
    toast.SetTag(&tag)?;

    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("com.yara.silo"))?;
    notifier.Show(&toast)?;

    Ok(())
}

fn trigger_native_toast(
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

    let title = match enforcement {
        "hard" => {
            if remaining_seconds <= 0 {
                "Focus Block: Closed".to_string()
            } else {
                format!("Hard Block in {}s", remaining_seconds)
            }
        }
        _ => {
            if remaining_seconds <= 0 {
                "Limit Reached".to_string()
            } else {
                let minutes = remaining_seconds / 60;
                if minutes > 0 {
                    format!("Remaining: {}m", minutes)
                } else {
                    format!("Remaining: {}s", remaining_seconds)
                }
            }
        }
    };

    let body = match enforcement {
        "hard" => {
            if remaining_seconds <= 0 {
                format!("Time limit exceeded. {} has been closed.", target)
            } else {
                format!("Save your work immediately! {} is closing.", target)
            }
        }
        "warn" => {
            if remaining_seconds <= 0 {
                format!("Limit reached! Closed {}. Extend to keep using.", target)
            } else {
                format!(
                    "Warning: Focus limit is approaching for {}. Extend time to keep active.",
                    target
                )
            }
        }
        "soft" => {
            format!(
                "Time limit reached today. {} minimized to help you stay focused.",
                target
            )
        }
        _ => "".to_string(),
    };

    // Show overlay in all warning/block cases when limit is < 60s or 0
    // ONLY show the overlay if the active app is in fullscreen mode
    let is_fullscreen = app_handle.state::<AppState>().active_app().is_fullscreen;

    if is_fullscreen && remaining_seconds <= 60 {
        let is_hard_block =
            (enforcement == "hard" || enforcement == "warn") && remaining_seconds <= 0;
        show_overlay(app_handle, &title, &body, is_hard_block);
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(err) = show_native_toast(
            rule_id,
            target,
            enforcement,
            limit_seconds,
            remaining_seconds,
            &title,
            &body,
        ) {
            tracing::error!("failed to show native toast: {:?}", err);
        }
    }
}

fn show_overlay(app_handle: &tauri::AppHandle, title: &str, body: &str, is_hard_block: bool) {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    if let Some(window) = app_handle.get_webview_window("overlay") {
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = app_handle.emit(
            "overlay_update",
            serde_json::json!({
                "title": title,
                "body": body,
                "is_hard_block": is_hard_block
            }),
        );
        return;
    }

    let url = format!(
        "overlay?title={}&body={}&hard={}",
        urlencoding::encode(title),
        urlencoding::encode(body),
        is_hard_block
    );

    let mut builder = WebviewWindowBuilder::new(app_handle, "overlay", WebviewUrl::App(url.into()))
        .title("SILO Overlay")
        .always_on_top(true)
        .inner_size(420.0, 120.0)
        .transparent(true)
        .skip_taskbar(true)
        .resizable(false)
        .closable(true);

    #[cfg(target_os = "windows")]
    {
        builder = builder.decorations(false);
    }

    if let Ok(window) = builder.build() {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let scale_factor = monitor.scale_factor();
            let screen_size = monitor.size();
            let logical_size = tauri::LogicalSize::new(420.0, 120.0);
            let physical_size = logical_size.to_physical::<u32>(scale_factor);

            let x = screen_size.width.saturating_sub(physical_size.width + 20);
            let y = screen_size.height.saturating_sub(physical_size.height + 60);
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
        let _ = window.show();
    }
}

fn redirect_active_tab(url: &str, is_fullscreen: bool) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let keys = if is_fullscreen {
            r#"$w.SendKeys('{ESC}'); Start-Sleep -Milliseconds 100; $w.SendKeys('{F11}'); Start-Sleep -Milliseconds 100;"#
        } else {
            ""
        };
        let script = format!(
            r#"$old = Get-Clipboard -Format Text -ErrorAction SilentlyContinue; Set-Clipboard -Value '{}'; $w = New-Object -ComObject WScript.Shell; {}$w.SendKeys('^l'); Start-Sleep -Milliseconds 100; $w.SendKeys('^v'); Start-Sleep -Milliseconds 100; $w.SendKeys('~'); Start-Sleep -Milliseconds 400; if ($old) {{ Set-Clipboard -Value $old }} else {{ Clear-Clipboard }}"#,
            url, keys
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .spawn();
    }
}

fn ensure_blocked_html(data_dir: &std::path::Path) -> std::io::Result<()> {
    let html_content = include_str!("blocked.html");
    let blocked_html_path = data_dir.join("blocked.html");
    std::fs::write(blocked_html_path, html_content)?;
    Ok(())
}

fn check_and_enforce_rules(
    app_handle: &tauri::AppHandle,
    state: &AppState,
    active_app: &models::ActiveApp,
) -> anyhow::Result<()> {
    let blocked_path = state.storage().data_dir().join("blocked.html");
    let blocked_url = format!(
        "file:///{}",
        blocked_path.to_string_lossy().replace('\\', "/")
    );
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
                    crate::monitor::normalize_domain(active_site)
                        == crate::monitor::normalize_domain(&rule.target)
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
            "site" => state.storage().today_site_seconds(&rule.target)?,
            _ => 0,
        };

        let limit = if rule.extra_limit_date.as_ref() == Some(&today_str) {
            rule.limit_seconds + rule.extra_limit_seconds
        } else {
            rule.limit_seconds
        };

        let remaining = limit - elapsed;
        let rule_state = rule_states.entry(rule_id).or_default();

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

                    if (remaining == 60 || remaining == 30 || remaining == 10)
                        && rule_state.last_countdown_sec != Some(remaining)
                    {
                        rule_state.last_countdown_sec = Some(remaining);
                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "hard",
                            limit,
                            remaining,
                        );
                    }
                }

                if matches_active && remaining <= 0 {
                    let now_ts = chrono::Utc::now().timestamp();
                    let should_track_block = match rule_state.last_blocked_ts {
                        Some(ts) => now_ts - ts >= 5,
                        None => true,
                    };
                    if should_track_block {
                        rule_state.last_blocked_ts = Some(now_ts);
                        let _ = state.storage().increment_rule_blocked(rule_id);
                    }

                    if rule.rule_type == "site" {
                        redirect_active_tab(&blocked_url, active_app.is_fullscreen);
                    } else {
                        close_target_app(active_app.pid);
                    }

                    if !rule_state.warned_soft_hit {
                        rule_state.warned_soft_hit = true;

                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "hard",
                            limit,
                            remaining,
                        );

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

                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "warn",
                            limit,
                            remaining,
                        );

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

                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "warn",
                            limit,
                            remaining,
                        );

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
                        if rule_state.last_countdown_sec != Some(remaining) {
                            rule_state.last_countdown_sec = Some(remaining);
                            if remaining == 60 || remaining == 30 || remaining == 10 {
                                trigger_native_toast(
                                    app_handle,
                                    rule_id,
                                    &rule.target,
                                    "warn",
                                    limit,
                                    remaining,
                                );
                            }
                        }

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
                    let now_ts = chrono::Utc::now().timestamp();
                    let should_track_block = match rule_state.last_blocked_ts {
                        Some(ts) => now_ts - ts >= 5,
                        None => true,
                    };
                    if should_track_block {
                        rule_state.last_blocked_ts = Some(now_ts);
                        let _ = state.storage().increment_rule_blocked(rule_id);
                    }

                    if rule.rule_type == "site" {
                        redirect_active_tab(&blocked_url, active_app.is_fullscreen);
                    } else {
                        close_target_app(active_app.pid);
                    }

                    if !rule_state.warned_soft_hit {
                        rule_state.warned_soft_hit = true;

                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "warn",
                            limit,
                            remaining,
                        );

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
                    let now_ts = chrono::Utc::now().timestamp();
                    let should_track_block = match rule_state.last_blocked_ts {
                        Some(ts) => now_ts - ts >= 5,
                        None => true,
                    };
                    if should_track_block {
                        rule_state.last_blocked_ts = Some(now_ts);
                        let _ = state.storage().increment_rule_blocked(rule_id);
                    }

                    minimize_active_window();

                    let now_ts = chrono::Utc::now().timestamp();
                    let should_notify = match rule_state.last_soft_notification_ts {
                        Some(ts) => now_ts - ts >= 10,
                        None => true,
                    };

                    if should_notify {
                        rule_state.last_soft_notification_ts = Some(now_ts);

                        trigger_native_toast(
                            app_handle,
                            rule_id,
                            &rule.target,
                            "soft",
                            limit,
                            remaining,
                        );

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

#[tauri::command]
fn close_overlay(app_handle: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app_handle.get_webview_window("overlay") {
        let _ = window.close();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_target(false).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(arg) = argv.get(1) {
                if arg.starts_with("silo://add-time/") {
                    let parts: Vec<&str> = arg
                        .strip_prefix("silo://add-time/")
                        .unwrap()
                        .split('/')
                        .collect();
                    if parts.len() == 2 {
                        if let (Ok(rule_id), Ok(seconds)) =
                            (parts[0].parse::<i64>(), parts[1].parse::<i64>())
                        {
                            let _ = extend_rule_limit(app, rule_id, seconds);
                        }
                    }
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let _ = register_custom_protocol();
            }

            let state = AppState::new()?;

            // Synchronize launch at startup preference
            use tauri_plugin_autostart::ManagerExt;
            let autostart_manager = app.autolaunch();
            if let Ok(settings) = state.storage().settings() {
                if settings.auto_start {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }
            }

            let data_dir = state.storage().data_dir();
            if let Err(e) = ensure_blocked_html(data_dir) {
                tracing::error!("failed to write blocked.html: {:?}", e);
            }
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
            let toggle_focus_i =
                MenuItemBuilder::with_id("toggle_focus", toggle_focus_text).build(app)?;
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
                .on_menu_event(move |app, event| match event.id.as_ref() {
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
                        let _ = app.emit(
                            "focus_mode_changed",
                            serde_json::json!({ "enabled": enabled }),
                        );

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
            add_rule_time,
            get_usage_range,
            get_rule_stats,
            close_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
