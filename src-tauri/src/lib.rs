mod api;
mod models;
mod monitor;
mod storage;

use api::{
    delete_rule, export_logs, export_usage_data, get_app_state, get_data_usage, get_network_speed,
    get_rules, get_settings, get_usage, get_usage_90d, handshake, mark_backup_complete, save_rule,
    save_settings, start_focus_mode, stop_focus_mode, toggle_focus_mode,
};
use models::AppReadyEvent;
use monitor::Monitor;
use std::time::Duration;
use storage::Storage;
use tauri::{Emitter, Manager};

pub struct AppState {
    storage: Storage,
    monitor: Monitor,
    focus_mode: parking_lot::Mutex<bool>,
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            storage: Storage::open_default()?,
            monitor: Monitor::new(),
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
            let today_seconds = state.storage().today_seconds().unwrap_or_default();

            let _ = app_handle.emit("update_active_app", &active_app);
            let _ = app_handle.emit(
                "usage_update",
                serde_json::json!({
                    "app": active_app.app,
                    "todaySeconds": today_seconds
                }),
            );
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt().with_target(false).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = AppState::new()?;
            app.manage(state);
            spawn_monitoring(app.handle().clone());
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
            mark_backup_complete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
