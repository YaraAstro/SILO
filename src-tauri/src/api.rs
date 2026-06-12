use crate::{
    models::{
        AppSnapshot, BootStatus, DataUsageReport, ExportResult, NetworkSpeed, Rule, Settings,
    },
    AppState,
};
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub fn handshake(state: State<'_, AppState>) -> CommandResult<BootStatus> {
    Ok(BootStatus {
        app_name: "SILO".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_ready: state.storage().database_ready(),
        focus_mode: state.focus_mode_enabled(),
        settings: state.storage().settings().map_err(to_command_error)?,
    })
}

#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>) -> CommandResult<AppSnapshot> {
    Ok(AppSnapshot {
        focus_mode: state.focus_mode_enabled(),
        active_app: state.active_app(),
        today_seconds: state.storage().today_seconds().map_err(to_command_error)?,
        rules_summary: state.storage().rules_summary().map_err(to_command_error)?,
        network_speed: state.network_speed(),
    })
}

#[tauri::command]
pub fn start_focus_mode(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.set_focus_mode(true);
    app.emit("focus_mode_changed", FocusModePayload { enabled: true })
        .map_err(to_command_error)
}

#[tauri::command]
pub fn stop_focus_mode(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.set_focus_mode(false);
    app.emit("focus_mode_changed", FocusModePayload { enabled: false })
        .map_err(to_command_error)
}

#[tauri::command]
pub fn toggle_focus_mode(app: AppHandle, state: State<'_, AppState>) -> CommandResult<bool> {
    let enabled = !state.focus_mode_enabled();
    state.set_focus_mode(enabled);
    app.emit("focus_mode_changed", FocusModePayload { enabled })
        .map_err(to_command_error)?;
    Ok(enabled)
}

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>) -> CommandResult<Vec<Rule>> {
    state.storage().rules().map_err(to_command_error)
}

#[tauri::command]
pub fn save_rule(app: AppHandle, state: State<'_, AppState>, rule: Rule) -> CommandResult<Rule> {
    let saved = state.storage().save_rule(rule).map_err(to_command_error)?;
    app.emit("rules_changed", &saved)
        .map_err(to_command_error)?;
    Ok(saved)
}

#[tauri::command]
pub fn delete_rule(app: AppHandle, state: State<'_, AppState>, id: i64) -> CommandResult<()> {
    state.storage().delete_rule(id).map_err(to_command_error)?;
    app.emit("rules_changed", DeletedRulePayload { id })
        .map_err(to_command_error)
}

#[tauri::command]
pub fn get_usage(
    state: State<'_, AppState>,
    date: String,
) -> CommandResult<crate::models::UsageReport> {
    state
        .storage()
        .usage_report(&date)
        .map_err(to_command_error)
}

#[tauri::command]
pub fn get_usage_90d(state: State<'_, AppState>) -> CommandResult<crate::models::UsageTimeline> {
    state.storage().usage_90d().map_err(to_command_error)
}

#[tauri::command]
pub fn get_network_speed(state: State<'_, AppState>) -> CommandResult<NetworkSpeed> {
    Ok(state.network_speed())
}

#[tauri::command]
pub fn get_data_usage(state: State<'_, AppState>, range: String) -> CommandResult<DataUsageReport> {
    state.storage().data_usage(&range).map_err(to_command_error)
}

#[tauri::command]
pub fn export_logs(state: State<'_, AppState>, range: String) -> CommandResult<ExportResult> {
    let payload = ExportPayload {
        exported_at: Utc::now().to_rfc3339(),
        export_type: "logs",
        range,
        note: "Structured log export is scaffolded; log rotation arrives in a later milestone.",
    };
    let file_path = state
        .storage()
        .export_json("logs", &payload)
        .map_err(to_command_error)?;
    Ok(ExportResult {
        file_path: file_path.display().to_string(),
    })
}

#[tauri::command]
pub fn export_usage_data(state: State<'_, AppState>, range: String) -> CommandResult<ExportResult> {
    let payload = state
        .storage()
        .data_usage(&range)
        .map_err(to_command_error)?;
    let file_path = state
        .storage()
        .export_json("usage", &payload)
        .map_err(to_command_error)?;
    Ok(ExportResult {
        file_path: file_path.display().to_string(),
    })
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<Settings> {
    state.storage().settings().map_err(to_command_error)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> CommandResult<Settings> {
    let saved = state
        .storage()
        .save_settings(&settings)
        .map_err(to_command_error)?;
    app.emit("settings_changed", &saved)
        .map_err(to_command_error)?;
    Ok(saved)
}

#[tauri::command]
pub fn mark_backup_complete(state: State<'_, AppState>) -> CommandResult<Settings> {
    state
        .storage()
        .mark_backup_complete()
        .map_err(to_command_error)
}

#[tauri::command]
pub fn get_available_apps(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let mut apps = std::collections::HashSet::new();

    if let Ok(tracked) = state.storage().get_tracked_apps() {
        for app in tracked {
            apps.insert(app);
        }
    }

    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for process in sys.processes().values() {
        let name = process.name().to_string_lossy().to_string();
        if !name.trim().is_empty() {
            apps.insert(name);
        }
    }

    let mut sorted_apps: Vec<String> = apps.into_iter().collect();
    sorted_apps.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    Ok(sorted_apps)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FocusModePayload {
    enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeletedRulePayload {
    id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPayload<'a> {
    exported_at: String,
    export_type: &'a str,
    range: String,
    note: &'a str,
}

fn to_command_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
