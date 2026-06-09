use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppReadyEvent {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootStatus {
    pub app_name: String,
    pub version: String,
    pub database_ready: bool,
    pub focus_mode: bool,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub focus_mode: bool,
    pub active_app: ActiveApp,
    pub today_seconds: i64,
    pub rules_summary: RulesSummary,
    pub network_speed: NetworkSpeed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveApp {
    pub app: String,
    pub title: String,
    pub elapsed_seconds: i64,
    pub pid: Option<u32>,
    pub sampled_at: i64,
}

impl Default for ActiveApp {
    fn default() -> Self {
        Self {
            app: "SILO".to_string(),
            title: "Monitoring engine pending".to_string(),
            elapsed_seconds: 0,
            pid: None,
            sampled_at: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesSummary {
    pub total: i64,
    pub active: i64,
    pub app_rules: i64,
    pub site_rules: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSpeed {
    pub upload_bps: i64,
    pub download_bps: i64,
}

impl Default for NetworkSpeed {
    fn default() -> Self {
        Self {
            upload_bps: 0,
            download_bps: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: Option<i64>,
    pub rule_type: String,
    pub target: String,
    pub limit_seconds: i64,
    pub enforcement: String,
    pub active: bool,
    pub schedule: Option<String>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_start: bool,
    pub theme: String,
    pub retention_days: i64,
    pub notifications_enabled: bool,
    pub sample_interval_seconds: i64,
    pub last_backup_at: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start: false,
            theme: "system".to_string(),
            retention_days: 90,
            notifications_enabled: true,
            sample_interval_seconds: 5,
            last_backup_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub date: String,
    pub total_seconds: i64,
    pub apps: Vec<UsageBucket>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTimeline {
    pub days: Vec<UsageDay>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageDay {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    pub name: String,
    pub seconds: i64,
}

#[derive(Debug, Clone)]
pub struct CompletedSession {
    pub app_name: String,
    pub window_title: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataUsageReport {
    pub range: String,
    pub apps: Vec<DataConsumer>,
    pub sites: Vec<DataConsumer>,
    pub total_upload_bytes: i64,
    pub total_download_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataConsumer {
    pub name: String,
    pub upload_bytes: i64,
    pub download_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub file_path: String,
}
