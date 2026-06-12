use crate::models::{
    CompletedSession, DataConsumer, DataUsageReport, Rule, RulesSummary, Settings, UsageBucket,
    UsageDay, UsageReport, UsageTimeline,
};
use anyhow::Context;
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const CURRENT_SCHEMA_VERSION: i64 = 1;

pub struct Storage {
    connection: Mutex<Connection>,
    data_dir: PathBuf,
}

impl Storage {
    pub fn open_default() -> anyhow::Result<Self> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("SILO");
        Self::open(data_dir)
    }

    pub fn open(data_dir: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("failed to create data dir {}", data_dir.display()))?;
        let db_path = data_dir.join("silo.sqlite");
        let connection = Connection::open(&db_path)
            .with_context(|| format!("failed to open sqlite db {}", db_path.display()))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;

        let storage = Self {
            connection: Mutex::new(connection),
            data_dir,
        };
        storage.migrate()?;
        storage.seed_settings()?;
        Ok(storage)
    }

    pub fn database_ready(&self) -> bool {
        self.connection
            .lock()
            .query_row("SELECT version FROM schema_migrations LIMIT 1", [], |_| {
                Ok(())
            })
            .is_ok()
    }

    pub fn settings(&self) -> anyhow::Result<Settings> {
        let conn = self.connection.lock();
        let mut settings = Settings::default();

        if let Some(value) = setting_value(&conn, "auto_start")? {
            settings.auto_start = value == "true";
        }
        if let Some(value) = setting_value(&conn, "theme")? {
            settings.theme = value;
        }
        if let Some(value) = setting_value(&conn, "retention_days")? {
            settings.retention_days = value.parse().unwrap_or(settings.retention_days);
        }
        if let Some(value) = setting_value(&conn, "notifications_enabled")? {
            settings.notifications_enabled = value == "true";
        }
        if let Some(value) = setting_value(&conn, "sample_interval_seconds")? {
            settings.sample_interval_seconds =
                value.parse().unwrap_or(settings.sample_interval_seconds);
        }
        settings.last_backup_at = setting_value(&conn, "last_backup_at")?;

        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> anyhow::Result<Settings> {
        let conn = self.connection.lock();
        upsert_setting(&conn, "auto_start", &settings.auto_start.to_string())?;
        upsert_setting(&conn, "theme", &settings.theme)?;
        upsert_setting(
            &conn,
            "retention_days",
            &settings.retention_days.to_string(),
        )?;
        upsert_setting(
            &conn,
            "notifications_enabled",
            &settings.notifications_enabled.to_string(),
        )?;
        upsert_setting(
            &conn,
            "sample_interval_seconds",
            &settings.sample_interval_seconds.to_string(),
        )?;
        if let Some(last_backup_at) = &settings.last_backup_at {
            upsert_setting(&conn, "last_backup_at", last_backup_at)?;
        }
        Ok(settings.clone())
    }

    pub fn mark_backup_complete(&self) -> anyhow::Result<Settings> {
        let mut settings = self.settings()?;
        settings.last_backup_at = Some(Utc::now().to_rfc3339());
        self.save_settings(&settings)
    }

    pub fn rules(&self) -> anyhow::Result<Vec<Rule>> {
        let conn = self.connection.lock();
        let mut statement = conn.prepare(
            "SELECT id, type, target, limit_seconds, enforcement, active, schedule, created_at, updated_at
             FROM rules
             ORDER BY active DESC, updated_at DESC, target ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(Rule {
                id: Some(row.get(0)?),
                rule_type: row.get(1)?,
                target: row.get(2)?,
                limit_seconds: row.get(3)?,
                enforcement: row.get(4)?,
                active: row.get::<_, i64>(5)? == 1,
                schedule: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn save_rule(&self, mut rule: Rule) -> anyhow::Result<Rule> {
        normalize_rule(&mut rule)?;

        let conn = self.connection.lock();
        let now = Utc::now().timestamp();
        match rule.id {
            Some(id) => {
                conn.execute(
                    "UPDATE rules
                     SET type = ?1, target = ?2, limit_seconds = ?3, enforcement = ?4,
                         active = ?5, schedule = ?6, updated_at = ?7
                     WHERE id = ?8",
                    params![
                        &rule.rule_type,
                        &rule.target,
                        rule.limit_seconds,
                        &rule.enforcement,
                        bool_to_int(rule.active),
                        &rule.schedule,
                        now,
                        id
                    ],
                )?;
                rule.updated_at = now;
                Ok(rule)
            }
            None => {
                conn.execute(
                    "INSERT INTO rules
                     (type, target, limit_seconds, enforcement, active, schedule, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        &rule.rule_type,
                        &rule.target,
                        rule.limit_seconds,
                        &rule.enforcement,
                        bool_to_int(rule.active),
                        &rule.schedule,
                        now,
                        now
                    ],
                )?;
                rule.id = Some(conn.last_insert_rowid());
                rule.created_at = now;
                rule.updated_at = now;
                Ok(rule)
            }
        }
    }

    pub fn delete_rule(&self, id: i64) -> anyhow::Result<()> {
        self.connection
            .lock()
            .execute("DELETE FROM rules WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_completed_session(&self, session: &CompletedSession) -> anyhow::Result<()> {
        self.connection.lock().execute(
            "INSERT INTO sessions
             (app_name, window_title, start_ts, end_ts, duration_seconds, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &session.app_name,
                &session.window_title,
                session.start_ts,
                session.end_ts,
                session.duration_seconds,
                Utc::now().timestamp()
            ],
        )?;
        Ok(())
    }

    pub fn rules_summary(&self) -> anyhow::Result<RulesSummary> {
        let conn = self.connection.lock();
        let total = count_rules(&conn, None, None)?;
        let active = count_rules(&conn, None, Some(true))?;
        let app_rules = count_rules(&conn, Some("app"), None)?;
        let site_rules = count_rules(&conn, Some("site"), None)?;

        Ok(RulesSummary {
            total,
            active,
            app_rules,
            site_rules,
        })
    }

    pub fn usage_report(&self, date: &str) -> anyhow::Result<UsageReport> {
        let conn = self.connection.lock();
        let day_start = format!("{date}T00:00:00Z");
        let day_end = format!("{date}T23:59:59Z");
        let mut statement = conn.prepare(
            "SELECT app_name, COALESCE(SUM(duration_seconds), 0)
             FROM sessions
             WHERE datetime(start_ts, 'unixepoch') BETWEEN datetime(?1) AND datetime(?2)
             GROUP BY app_name
             ORDER BY SUM(duration_seconds) DESC",
        )?;
        let rows = statement.query_map(params![day_start, day_end], |row| {
            Ok(UsageBucket {
                name: row.get(0)?,
                seconds: row.get(1)?,
            })
        })?;
        let apps = rows.collect::<Result<Vec<_>, _>>()?;
        let total_seconds = apps.iter().map(|bucket| bucket.seconds).sum();

        Ok(UsageReport {
            date: date.to_string(),
            total_seconds,
            apps,
        })
    }

    pub fn today_seconds(&self) -> anyhow::Result<i64> {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        Ok(self.usage_report(&today)?.total_seconds)
    }

    pub fn usage_90d(&self) -> anyhow::Result<UsageTimeline> {
        let start = Utc::now().date_naive() - Duration::days(89);
        let conn = self.connection.lock();
        let mut statement = conn.prepare(
            "SELECT date(datetime(start_ts, 'unixepoch')) AS usage_date,
                    COALESCE(SUM(duration_seconds), 0)
             FROM sessions
             WHERE date(datetime(start_ts, 'unixepoch')) >= ?1
             GROUP BY usage_date
             ORDER BY usage_date ASC",
        )?;
        let rows = statement.query_map(params![start.to_string()], |row| {
            Ok(UsageDay {
                date: row.get(0)?,
                total_seconds: row.get(1)?,
            })
        })?;

        Ok(UsageTimeline {
            days: rows.collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn data_usage(&self, range: &str) -> anyhow::Result<DataUsageReport> {
        let days = match range {
            "7d" => 7,
            "30d" => 30,
            "90d" => 90,
            _ => 30,
        };
        let start = (Utc::now().date_naive() - Duration::days(days - 1)).to_string();
        let conn = self.connection.lock();
        let apps = data_consumers(
            &conn,
            "app_data_usage",
            "app_name",
            "date >= ?1 GROUP BY app_name ORDER BY SUM(upload_bytes + download_bytes) DESC",
            &start,
        )?;
        let sites = data_consumers(
            &conn,
            "site_data_usage",
            "domain",
            "date >= ?1 GROUP BY domain ORDER BY SUM(upload_bytes + download_bytes) DESC",
            &start,
        )?;
        let total_upload_bytes = apps.iter().map(|item| item.upload_bytes).sum::<i64>()
            + sites.iter().map(|item| item.upload_bytes).sum::<i64>();
        let total_download_bytes = apps.iter().map(|item| item.download_bytes).sum::<i64>()
            + sites.iter().map(|item| item.download_bytes).sum::<i64>();

        Ok(DataUsageReport {
            range: range.to_string(),
            apps,
            sites,
            total_upload_bytes,
            total_download_bytes,
        })
    }

    pub fn export_json<T: Serialize>(&self, prefix: &str, payload: &T) -> anyhow::Result<PathBuf> {
        let export_dir = self.data_dir.join("exports");
        fs::create_dir_all(&export_dir)?;
        let file_path = export_dir.join(format!(
            "{prefix}-{}-{}.json",
            Utc::now().format("%Y%m%d%H%M%S"),
            Uuid::new_v4()
        ));
        let json = serde_json::to_string_pretty(payload)?;
        fs::write(&file_path, json)?;
        Ok(file_path)
    }

    fn migrate(&self) -> anyhow::Result<()> {
        let conn = self.connection.lock();
        conn.execute_batch(include_str!("../schema/001_initial.sql"))?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![CURRENT_SCHEMA_VERSION, Utc::now().timestamp()],
        )?;
        Ok(())
    }

    fn seed_settings(&self) -> anyhow::Result<()> {
        let defaults = Settings::default();
        let conn = self.connection.lock();
        insert_setting_if_missing(&conn, "auto_start", &defaults.auto_start.to_string())?;
        insert_setting_if_missing(&conn, "theme", &defaults.theme)?;
        insert_setting_if_missing(
            &conn,
            "retention_days",
            &defaults.retention_days.to_string(),
        )?;
        insert_setting_if_missing(
            &conn,
            "notifications_enabled",
            &defaults.notifications_enabled.to_string(),
        )?;
        insert_setting_if_missing(
            &conn,
            "sample_interval_seconds",
            &defaults.sample_interval_seconds.to_string(),
        )?;
        Ok(())
    }

    pub fn get_tracked_apps(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.connection.lock();
        let mut statement = conn.prepare(
            "SELECT DISTINCT app_name FROM sessions 
             UNION 
             SELECT DISTINCT app_name FROM app_data_usage 
             ORDER BY app_name ASC"
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut apps = Vec::new();
        for app in rows {
            if let Ok(app) = app {
                let trimmed = app.trim();
                if !trimmed.is_empty() {
                    apps.push(trimmed.to_string());
                }
            }
        }
        Ok(apps)
    }

    pub fn track_site_time(&self, domain: &str, seconds: i64) -> anyhow::Result<()> {
        let conn = self.connection.lock();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let exists: Option<i64> = conn.query_row(
            "SELECT id FROM site_usage WHERE domain = ?1 AND date = ?2 LIMIT 1",
            params![domain, today],
            |row| row.get(0),
        ).optional()?;

        match exists {
            Some(id) => {
                conn.execute(
                    "UPDATE site_usage SET seconds = seconds + ?1 WHERE id = ?2",
                    params![seconds, id],
                )?;
            }
            None => {
                conn.execute(
                    "INSERT INTO site_usage (domain, date, seconds) VALUES (?1, ?2, ?3)",
                    params![domain, today, seconds],
                )?;
            }
        }
        Ok(())
    }

    pub fn today_site_seconds(&self, domain: &str) -> anyhow::Result<i64> {
        let conn = self.connection.lock();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let seconds: Option<i64> = conn.query_row(
            "SELECT seconds FROM site_usage WHERE domain = ?1 AND date = ?2",
            params![domain, today],
            |row| row.get(0),
        ).optional()?;
        Ok(seconds.unwrap_or(0))
    }

    pub fn today_app_seconds(&self, app_name: &str) -> anyhow::Result<i64> {
        let conn = self.connection.lock();
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let day_start = format!("{today}T00:00:00Z");
        let day_end = format!("{today}T23:59:59Z");
        let seconds: Option<i64> = conn.query_row(
            "SELECT COALESCE(SUM(duration_seconds), 0)
             FROM sessions
             WHERE app_name = ?1 AND datetime(start_ts, 'unixepoch') BETWEEN datetime(?2) AND datetime(?3)",
            params![app_name, day_start, day_end],
            |row| row.get(0),
        ).optional()?;
        Ok(seconds.unwrap_or(0))
    }
}

fn setting_value(conn: &Connection, key: &str) -> anyhow::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn upsert_setting(conn: &Connection, key: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn insert_setting_if_missing(conn: &Connection, key: &str, value: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

fn count_rules(
    conn: &Connection,
    rule_type: Option<&str>,
    active: Option<bool>,
) -> anyhow::Result<i64> {
    let (sql, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = match (rule_type, active) {
        (Some(rule_type), Some(active)) => (
            "SELECT COUNT(*) FROM rules WHERE type = ?1 AND active = ?2",
            vec![
                Box::new(rule_type.to_string()),
                Box::new(bool_to_int(active)),
            ],
        ),
        (Some(rule_type), None) => (
            "SELECT COUNT(*) FROM rules WHERE type = ?1",
            vec![Box::new(rule_type.to_string())],
        ),
        (None, Some(active)) => (
            "SELECT COUNT(*) FROM rules WHERE active = ?1",
            vec![Box::new(bool_to_int(active))],
        ),
        (None, None) => ("SELECT COUNT(*) FROM rules", vec![]),
    };

    let params = params
        .iter()
        .map(|value| value.as_ref())
        .collect::<Vec<_>>();
    conn.query_row(sql, params.as_slice(), |row| row.get(0))
        .map_err(Into::into)
}

fn data_consumers(
    conn: &Connection,
    table: &str,
    name_column: &str,
    clause: &str,
    start: &str,
) -> anyhow::Result<Vec<DataConsumer>> {
    let sql = format!(
        "SELECT {name_column}, COALESCE(SUM(upload_bytes), 0), COALESCE(SUM(download_bytes), 0)
         FROM {table}
         WHERE {clause}
         LIMIT 25"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params![start], |row| {
        Ok(DataConsumer {
            name: row.get(0)?,
            upload_bytes: row.get(1)?,
            download_bytes: row.get(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn normalize_rule(rule: &mut Rule) -> anyhow::Result<()> {
    let rule_type = rule.rule_type.trim().to_ascii_lowercase();
    if !matches!(rule_type.as_str(), "app" | "site") {
        anyhow::bail!("rule_type must be app or site");
    }

    let enforcement = rule.enforcement.trim().to_ascii_lowercase();
    if !matches!(enforcement.as_str(), "soft" | "hard" | "warn") {
        anyhow::bail!("enforcement must be soft, hard, or warn");
    }

    let target = rule.target.trim();
    if target.is_empty() {
        anyhow::bail!("target is required");
    }

    rule.rule_type = rule_type;
    rule.enforcement = enforcement;
    rule.target = target.to_string();
    rule.limit_seconds = rule.limit_seconds.max(0);
    Ok(())
}

fn bool_to_int(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

#[allow(dead_code)]
fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
