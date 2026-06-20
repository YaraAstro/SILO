CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  type TEXT NOT NULL CHECK (type IN ('app', 'site')),
  target TEXT NOT NULL,
  limit_seconds INTEGER DEFAULT 0,
  enforcement TEXT NOT NULL CHECK (enforcement IN ('soft', 'hard', 'warn')),
  active INTEGER DEFAULT 1,
  schedule TEXT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  app_name TEXT NOT NULL,
  window_title TEXT,
  start_ts INTEGER NOT NULL,
  end_ts INTEGER,
  duration_seconds INTEGER,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS site_usage (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  domain TEXT NOT NULL,
  date TEXT NOT NULL,
  seconds INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS app_data_usage (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  app_name TEXT NOT NULL,
  date TEXT NOT NULL,
  upload_bytes INTEGER DEFAULT 0,
  download_bytes INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS site_data_usage (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  domain TEXT NOT NULL,
  date TEXT NOT NULL,
  upload_bytes INTEGER DEFAULT 0,
  download_bytes INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS network_samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sample_ts INTEGER NOT NULL,
  upload_bps INTEGER DEFAULT 0,
  download_bps INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT
);

CREATE TABLE IF NOT EXISTS logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  level TEXT NOT NULL,
  message TEXT NOT NULL,
  context TEXT,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rules_target ON rules(type, target);
CREATE INDEX IF NOT EXISTS idx_sessions_app_start ON sessions(app_name, start_ts);
CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_ts);
CREATE INDEX IF NOT EXISTS idx_site_usage_domain_date ON site_usage(domain, date);
CREATE INDEX IF NOT EXISTS idx_app_data_usage_app_date ON app_data_usage(app_name, date);
CREATE INDEX IF NOT EXISTS idx_site_data_usage_domain_date ON site_data_usage(domain, date);
CREATE INDEX IF NOT EXISTS idx_network_samples_ts ON network_samples(sample_ts);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON logs(created_at);

CREATE TABLE IF NOT EXISTS rule_stats (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_id INTEGER NOT NULL,
  date TEXT NOT NULL,
  times_blocked INTEGER DEFAULT 0,
  times_bypassed INTEGER DEFAULT 0,
  UNIQUE(rule_id, date)
);
CREATE INDEX IF NOT EXISTS idx_rule_stats_rule_date ON rule_stats(rule_id, date);
