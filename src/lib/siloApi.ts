import { invoke } from "@tauri-apps/api/core";

export type Settings = {
  autoStart: boolean;
  theme: "system" | "light" | "dark";
  retentionDays: number;
  notificationsEnabled: boolean;
  sampleIntervalSeconds: number;
  lastBackupAt: string | null;
  shortcutsEnabled: boolean;
};

export type Rule = {
  id: number | null;
  ruleType: "app" | "site";
  target: string;
  limitSeconds: number;
  enforcement: "soft" | "hard" | "warn";
  active: boolean;
  schedule: string | null;
  createdAt: number;
  updatedAt: number;
  extraLimitSeconds?: number;
  extraLimitDate?: string | null;
};

export type BootStatus = {
  appName: string;
  version: string;
  databaseReady: boolean;
  focusMode: boolean;
  settings: Settings;
};

export type AppSnapshot = {
  focusMode: boolean;
  activeApp: {
    app: string;
    title: string;
    elapsedSeconds: number;
    pid: number | null;
    sampledAt: number;
    site?: string;
  };
  todaySeconds: number;
  rulesSummary: {
    total: number;
    active: number;
    appRules: number;
    siteRules: number;
  };
  networkSpeed: NetworkSpeed;
};

export type NetworkSpeed = {
  uploadBps: number;
  downloadBps: number;
};

export type UsageReport = {
  date: string;
  totalSeconds: number;
  apps: Array<{ name: string; seconds: number }>;
  sites?: Array<{ name: string; seconds: number }>;
};

export type UsageTimeline = {
  days: Array<{ date: string; totalSeconds: number }>;
};

export type DataUsageReport = {
  range: string;
  apps: Array<DataConsumer>;
  sites: Array<DataConsumer>;
  totalUploadBytes: number;
  totalDownloadBytes: number;
};

export type DataConsumer = {
  name: string;
  uploadBytes: number;
  downloadBytes: number;
};

export type ExportResult = {
  filePath: string;
};

export type UsageDayBytes = {
  date: string;
  uploadBytes: number;
  downloadBytes: number;
};

export type RuleStats = {
  ruleId: number;
  date: string;
  timesBlocked: number;
  timesBypassed: number;
};

export const emptyRule = (): Rule => ({
  id: null,
  ruleType: "app",
  target: "",
  limitSeconds: 1800,
  enforcement: "soft",
  active: true,
  schedule: null,
  createdAt: 0,
  updatedAt: 0,
  extraLimitSeconds: 0,
  extraLimitDate: null,
});

export const siloApi = {
  handshake: () => invoke<BootStatus>("handshake"),
  getAppState: () => invoke<AppSnapshot>("get_app_state"),
  startFocusMode: () => invoke<void>("start_focus_mode"),
  stopFocusMode: () => invoke<void>("stop_focus_mode"),
  toggleFocusMode: () => invoke<boolean>("toggle_focus_mode"),
  getRules: () => invoke<Rule[]>("get_rules"),
  saveRule: (rule: Rule) => invoke<Rule>("save_rule", { rule }),
  deleteRule: (id: number) => invoke<void>("delete_rule", { id }),
  getUsage: (date: string) => invoke<UsageReport>("get_usage", { date }),
  getUsageRange: (range: string) => invoke<UsageReport>("get_usage_range", { range }),
  getUsage90d: () => invoke<UsageTimeline>("get_usage_90d"),
  getNetworkSpeed: () => invoke<NetworkSpeed>("get_network_speed"),
  getDataUsage: (range: string) => invoke<DataUsageReport>("get_data_usage", { range }),
  exportLogs: (range: string) => invoke<ExportResult>("export_logs", { range }),
  exportUsageData: (range: string) => invoke<ExportResult>("export_usage_data", { range }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  markBackupComplete: () => invoke<Settings>("mark_backup_complete"),
  getAvailableApps: () => invoke<string[]>("get_available_apps"),
  getNetworkHistory: (range: string) => invoke<UsageDayBytes[]>("get_network_history", { range }),
  addRuleTime: (id: number, seconds: number) => invoke<void>("add_rule_time", { id, seconds }),
  getRuleStats: (range: string) => invoke<RuleStats[]>("get_rule_stats", { range }),
};
