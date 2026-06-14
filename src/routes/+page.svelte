<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { fade, fly, slide, scale } from "svelte/transition";
  import {
    Activity,
    Award,
    Calendar,
    ChartColumn,
    CircleAlert,
    Clock,
    Database,
    Download,
    FileDown,
    Flame,
    Gauge,
    Globe,
    HardDrive,
    House,
    Info,
    Keyboard,
    Monitor,
    Moon,
    Plus,
    Power,
    RotateCcw,
    Search,
    Settings as SettingsIcon,
    Shield,
    Sparkles,
    Target,
    Timer,
    Trash2,
    Upload,
    Wifi,
  } from "lucide-svelte";
  import BottomNav from "$lib/components/BottomNav.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import MetricCard from "$lib/components/MetricCard.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";
  import TrendChart from "$lib/components/TrendChart.svelte";
  import {
    emptyRule,
    siloApi,
    type AppSnapshot,
    type BootStatus,
    type DataConsumer,
    type DataUsageReport,
    type Rule,
    type Settings,
    type UsageReport,
    type UsageTimeline,
    type UsageDayBytes,
  } from "$lib/siloApi";

  type ViewKey =
    | "dashboard"
    | "rules"
    | "stats"
    | "network"
    | "settings"
    | "network-apps"
    | "network-sites";
  type RangeKey = "7d" | "30d" | "90d";

  const navItems = [
    { key: "dashboard", label: "Dashboard", icon: House },
    { key: "rules", label: "Rules", icon: Shield },
    { key: "stats", label: "Stats", icon: ChartColumn },
    { key: "network", label: "Network", icon: Wifi },
    { key: "settings", label: "Settings", icon: SettingsIcon },
  ];

  let activeView = $state<ViewKey>("dashboard");
  let boot = $state<BootStatus | null>(null);
  let snapshot = $state<AppSnapshot | null>(null);
  let rules = $state<Rule[]>([]);
  let settings = $state<Settings | null>(null);
  let usage = $state<UsageReport | null>(null);
  let usageTab = $state<"apps" | "sites">("apps");
  let isRefreshing = $state(false);
  let refreshKey = $state(0);
  let timeline = $state<UsageTimeline | null>(null);
  let dataUsage = $state<DataUsageReport | null>(null);
  let ruleDraft = $state<Rule>(emptyRule());
  let dataRange = $state<RangeKey>("30d");
  
  // Enhanced Stats View State
  let statsSubTab = $state<"screentime" | "network" | "habits">("screentime");
  let statsRange = $state<"today" | RangeKey>("7d");
  let statsScreenTab = $state<"apps" | "sites">("apps");
  let statsNetworkTab = $state<"apps" | "sites">("apps");
  let statsSearch = $state("");
  let screenTimePage = $state(1);
  let networkPage = $state(1);
  let statsUsage = $state<UsageReport | null>(null);
  let statsDataUsage = $state<DataUsageReport | null>(null);
  let statsNetworkHistory = $state<UsageDayBytes[]>([]);
  let heatmapNetworkHistory = $state<UsageDayBytes[]>([]);
  let heatmapTab = $state<"screentime" | "network">("screentime");
  let ruleSearch = $state("");
  let loading = $state(true);
  let savingRule = $state(false);
  let savingSettings = $state(false);
  let exporting = $state(false);
  let exportPath = $state("");
  let availableApps = $state<string[]>([]);
  let showAppDropdown = $state(false);
  let showSiteDropdown = $state(false);
  let showViolationOverlay = $state(false);
  let violationData = $state<{
    ruleId: number;
    target: string;
    ruleType: string;
    enforcement: string;
    limitSeconds: number;
    remainingSeconds: number;
  } | null>(null);

  let showCountdownOverlay = $state(false);
  let countdownData = $state<{
    ruleId: number;
    target: string;
    remainingSeconds: number;
    enforcement: string;
  } | null>(null);

  let showWarningOverlay = $state(false);
  let warningData = $state<{
    ruleId: number;
    target: string;
    ruleType: string;
    enforcement: string;
    limitSeconds: number;
    remainingSeconds: number;
  } | null>(null);

  let detailRange = $state<"today" | "7d" | "30d">("today");
  let detailUsage = $state<DataUsageReport | null>(null);
  let moreModalSearch = $state("");
  
  let showConfirmDeleteOverlay = $state(false);
  let ruleToDelete = $state<Rule | null>(null);
  let showRuleForm = $state(false);

  let filteredMoreRows = $derived(
    (activeView === "network-apps"
      ? (detailUsage?.apps ?? [])
      : (detailUsage?.sites ?? [])
    ).filter((row) =>
      row.name.toLowerCase().includes(moreModalSearch.toLowerCase()),
    ),
  );

  const pageSize = 5;

  let filteredStatsScreenList = $derived(
    (statsScreenTab === "apps" 
      ? (statsUsage?.apps ?? []) 
      : (statsUsage?.sites ?? [])
    ).filter(item => item.name.toLowerCase().includes(statsSearch.toLowerCase()))
  );

  let paginatedStatsScreenList = $derived(
    filteredStatsScreenList.slice((screenTimePage - 1) * pageSize, screenTimePage * pageSize)
  );

  let totalScreenPages = $derived(
    Math.max(1, Math.ceil(filteredStatsScreenList.length / pageSize))
  );

  let filteredStatsNetworkList = $derived(
    (statsNetworkTab === "apps" 
      ? (statsDataUsage?.apps ?? []) 
      : (statsDataUsage?.sites ?? [])
    ).filter(item => item.name.toLowerCase().includes(statsSearch.toLowerCase()))
  );

  let paginatedStatsNetworkList = $derived(
    filteredStatsNetworkList.slice((networkPage - 1) * pageSize, networkPage * pageSize)
  );

  let totalNetworkPages = $derived(
    Math.max(1, Math.ceil(filteredStatsNetworkList.length / pageSize))
  );

  // Dynamic Focus Score calculations
  let focusScoreValue = $derived.by(() => {
    let score = 100;
    
    rules.forEach(rule => {
      if (rule.active) {
        const remaining = getRuleRemainingSeconds(rule);
        const limit = rule.extraLimitDate === new Date().toISOString().slice(0, 10)
          ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
          : rule.limitSeconds;
        
        if (limit > 0) {
          const usedPct = ((limit - remaining) / limit) * 100;
          if (usedPct >= 100) {
            score -= 20; 
          } else if (usedPct >= 80) {
            score -= 10; 
          } else if (usedPct >= 50) {
            score -= 5;  
          }
        }
      }
    });

    return Math.max(20, Math.min(100, score));
  });

  let focusScoreLabel = $derived.by(() => {
    const score = focusScoreValue;
    if (score >= 90) return "Optimal Focus";
    if (score >= 75) return "Good Focus";
    if (score >= 60) return "Moderate distractions";
    return "Highly Distracted";
  });

  let focusScoreColor = $derived.by(() => {
    const val = focusScoreValue;
    if (val >= 90) return { primary: "#2dd4bf", secondary: "#14b8a6", shadow: "rgba(45,212,191,0.25)", label: "text-teal-400 border-teal-500/20 bg-teal-500/5" };
    if (val >= 75) return { primary: "#3b82f6", secondary: "#1d4ed8", shadow: "rgba(59,130,246,0.25)", label: "text-blue-400 border-blue-500/20 bg-blue-500/5" };
    if (val >= 60) return { primary: "#fbbf24", secondary: "#d97706", shadow: "rgba(251,191,36,0.25)", label: "text-amber-400 border-amber-500/20 bg-amber-500/5" };
    return { primary: "#f43f5e", secondary: "#be123c", shadow: "rgba(244,63,94,0.25)", label: "text-rose-400 border-rose-500/20 bg-rose-500/5" };
  });

  let greetingText = $derived.by(() => {
    const hour = new Date().getHours();
    if (hour < 12) return "Good Morning";
    if (hour < 17) return "Good Afternoon";
    if (hour < 22) return "Good Evening";
    return "Good Night";
  });

  let currentDateStr = $derived.by(() => {
    return new Date().toLocaleDateString("en-US", {
      weekday: "long",
      month: "short",
      day: "numeric"
    });
  });

  // Dynamic AI insights generator
  let aiInsightsList = $derived.by(() => {
    let list: Array<{ title: string; desc: string; type: "success" | "warn" | "info" }> = [];
    
    if (snapshot?.focusMode) {
      list.push({
        title: "Focus Shield Active",
        desc: "SILO is actively monitoring and shielding your session rules.",
        type: "success"
      });
    } else {
      list.push({
        title: "Shield Deactivated",
        desc: "Rule limits will not be blocked unless Focus Shield is activated.",
        type: "info"
      });
    }

    const totalSeconds = snapshot?.todaySeconds ?? 0;
    const hours = totalSeconds / 3600;
    if (hours > 6) {
      list.push({
        title: "High Workload Alert",
        desc: "You've tracked over 6 hours of work today. Take a quick break to refresh.",
        type: "warn"
      });
    } else if (hours > 2) {
      list.push({
        title: "Great Momentum",
        desc: "Logging solid productive hours today. Keep it up!",
        type: "success"
      });
    }

    let limitWarning = false;
    rules.forEach(rule => {
      if (rule.active) {
        const remaining = getRuleRemainingSeconds(rule);
        if (remaining > 0 && remaining <= 300) {
          limitWarning = true;
          list.push({
            title: `Approaching Limit: ${rule.target}`,
            desc: `Only ${formatDuration(remaining)} remaining before limit enforcement.`,
            type: "warn"
          });
        }
      }
    });

    if (!limitWarning && list.length < 3) {
      list.push({
        title: "Clean Record",
        desc: "All monitored services are within their allowed limits today.",
        type: "success"
      });
    }

    return list.slice(0, 3);
  });

  $effect(() => {
    statsSearch;
    statsScreenTab;
    statsNetworkTab;
    statsSubTab;
    statsRange;
    screenTimePage = 1;
    networkPage = 1;
  });

  async function loadDetailUsage(range: "today" | "7d" | "30d") {
    try {
      detailRange = range;
      detailUsage = await siloApi.getDataUsage(range);
    } catch (error) {
      console.error("Failed to load detail network usage:", error);
    }
  }

  async function openMoreScreen(title: string, rows: DataConsumer[]) {
    moreModalSearch = "";
    activeView = title === "Top Apps" ? "network-apps" : "network-sites";
    await loadDetailUsage("today");
  }

  function getDetailTotalBytes() {
    const list =
      activeView === "network-apps"
        ? (detailUsage?.apps ?? [])
        : (detailUsage?.sites ?? []);
    return list.reduce(
      (sum, item) => sum + item.downloadBytes + item.uploadBytes,
      0,
    );
  }

  function getDetailDownloadBytes() {
    const list =
      activeView === "network-apps"
        ? (detailUsage?.apps ?? [])
        : (detailUsage?.sites ?? []);
    return list.reduce((sum, item) => sum + item.downloadBytes, 0);
  }

  function getDetailUploadBytes() {
    const list =
      activeView === "network-apps"
        ? (detailUsage?.apps ?? [])
        : (detailUsage?.sites ?? []);
    return list.reduce((sum, item) => sum + item.uploadBytes, 0);
  }

  let liveNetworkSamples = $state<
    Array<{ time: string; down: number; up: number }>
  >([]);
  let networkHistoryRange = $state<"7d" | "30d">("7d");
  let networkHistory = $state<UsageDayBytes[]>([]);

  let touchStartX = 0;
  let touchStartY = 0;

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
  }

  function handleTouchEnd(e: TouchEvent) {
    const diffX = e.changedTouches[0].clientX - touchStartX;
    const diffY = e.changedTouches[0].clientY - touchStartY;

    if (Math.abs(diffX) > 80 && Math.abs(diffY) < 50) {
      const views: ViewKey[] = [
        "dashboard",
        "rules",
        "stats",
        "network",
        "settings",
      ];
      const currentIndex = views.indexOf(activeView);
      if (diffX > 0) {
        if (currentIndex > 0) {
          activeView = views[currentIndex - 1];
        }
      } else {
        if (currentIndex < views.length - 1) {
          activeView = views[currentIndex + 1];
        }
      }
    }
  }

  let lastSwipeTime = 0;

  function handleWheel(e: WheelEvent) {
    if (Math.abs(e.deltaX) > 35) {
      const now = Date.now();
      if (now - lastSwipeTime < 800) {
        return;
      }

      const views: ViewKey[] = [
        "dashboard",
        "rules",
        "stats",
        "network",
        "settings",
      ];
      const currentIndex = views.indexOf(activeView);

      if (e.deltaX < 0) {
        if (currentIndex > 0) {
          activeView = views[currentIndex - 1];
          lastSwipeTime = now;
        }
      } else {
        if (currentIndex < views.length - 1) {
          activeView = views[currentIndex + 1];
          lastSwipeTime = now;
        }
      }
    }
  }

  interface Toast {
    id: string;
    message: string;
    type: "success" | "error" | "info";
  }

  let toasts = $state<Toast[]>([]);

  function showToast(
    message: string,
    type: "success" | "error" | "info" = "info",
  ) {
    const id = Math.random().toString();
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => {
      toasts = toasts.filter((t) => t.id !== id);
    }, 4000);

    if (settings?.notificationsEnabled) {
      if (typeof window !== "undefined" && "Notification" in window) {
        if (Notification.permission === "granted") {
          new Notification("SILO", { body: message });
        } else if (Notification.permission !== "denied") {
          Notification.requestPermission().then((permission) => {
            if (permission === "granted") {
              new Notification("SILO", { body: message });
            }
          });
        }
      }
    }
  }

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    void loadAll();
    const timer = window.setInterval(() => void refreshLiveState(), 5000);

    void listen<AppSnapshot["activeApp"]>("update_active_app", (event) => {
      if (snapshot) snapshot = { ...snapshot, activeApp: event.payload };
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{ app: string; todaySeconds: number }>(
      "usage_update",
      (event) => {
        if (snapshot)
          snapshot = { ...snapshot, todaySeconds: event.payload.todaySeconds };
        void loadTodayUsage();
      },
    )
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{ enabled: boolean }>("focus_mode_changed", (event) => {
      if (snapshot)
        snapshot = { ...snapshot, focusMode: event.payload.enabled };
      if (boot) boot = { ...boot, focusMode: event.payload.enabled };
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{
      ruleId: number;
      target: string;
      ruleType: string;
      enforcement: string;
      limitSeconds: number;
      remainingSeconds: number;
    }>("rule_violated", (event) => {
      violationData = event.payload;
      showViolationOverlay = true;
      showCountdownOverlay = false;
      showWarningOverlay = false;
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{
      ruleId: number;
      target: string;
      remainingSeconds: number;
      enforcement: string;
    }>("rule_countdown", (event) => {
      countdownData = event.payload;
      showCountdownOverlay = true;
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{
      ruleId: number;
      target: string;
      ruleType: string;
      enforcement: string;
      limitSeconds: number;
      remainingSeconds: number;
    }>("rule_warning", (event) => {
      warningData = event.payload;
      showWarningOverlay = true;
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<string>("navigate", (event) => {
      activeView = event.payload as ViewKey;
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    return () => {
      window.clearInterval(timer);
      unlisteners.forEach((unlisten) => unlisten());
    };
  });

  async function loadAll() {
    loading = true;
    try {
      boot = await siloApi.handshake();
      settings = boot.settings;
      await Promise.all([
        loadSnapshot(),
        loadRules(),
        loadTodayUsage(),
        loadTimeline(),
        loadDataUsage(dataRange),
        loadAvailableApps(),
        loadNetworkHistory(),
        loadStatsData(statsRange),
      ]);
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      loading = false;
    }
  }

  async function loadAvailableApps() {
    try {
      availableApps = await siloApi.getAvailableApps();
    } catch (error) {
      console.error("Failed to load available apps:", error);
    }
  }

  function filteredAvailableApps() {
    const query = ruleDraft.target.trim().toLowerCase();
    if (!query) return availableApps;
    return availableApps.filter((app) => app.toLowerCase().includes(query));
  }

  function filteredAvailableSites() {
    const sitesList = (usage?.sites ?? []).map((s) => s.name);
    const query = ruleDraft.target.trim().toLowerCase();
    if (!query) return sitesList;
    return sitesList.filter((site) => site.toLowerCase().includes(query));
  }

  async function refreshLiveState() {
    await Promise.all([
      loadSnapshot(),
      loadDataUsage(dataRange),
      loadNetworkHistory(),
    ]).catch((error) => {
      showToast(toErrorMessage(error), "error");
    });

    const nowTime = new Date().toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
    const downloadMb = (snapshot?.networkSpeed.downloadBps ?? 0) / 1_048_576;
    const uploadMb = (snapshot?.networkSpeed.uploadBps ?? 0) / 1_048_576;
    liveNetworkSamples = [
      ...liveNetworkSamples,
      { time: nowTime, down: downloadMb, up: uploadMb },
    ].slice(-60);
  }

  async function loadNetworkHistory() {
    try {
      networkHistory = await siloApi.getNetworkHistory(networkHistoryRange);
    } catch (error) {
      console.error("Failed to load network history:", error);
    }
  }

  async function changeNetworkHistoryRange(range: "7d" | "30d") {
    networkHistoryRange = range;
    await loadNetworkHistory();
  }

  function liveChartData() {
    return [
      {
        label: "Download (MB/s)",
        data: liveNetworkSamples.map((s) => s.down),
        backgroundColor: "rgba(20, 184, 166, 0.15)",
        borderColor: "#2dd4bf",
        borderWidth: 2,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
      },
      {
        label: "Upload (MB/s)",
        data: liveNetworkSamples.map((s) => s.up),
        backgroundColor: "rgba(139, 92, 246, 0.15)",
        borderColor: "#a78bfa",
        borderWidth: 2,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
      },
    ];
  }

  function liveChartLabels() {
    return liveNetworkSamples.map((s) => s.time);
  }

  function historyChartData() {
    return [
      {
        label: "Total Usage (MB)",
        data: networkHistory.map((day) =>
          Math.round((day.downloadBytes + day.uploadBytes) / 1_048_576),
        ),
        backgroundColor: "rgba(20, 184, 166, 0.85)",
        borderColor: "#2dd4bf",
        borderRadius: 6,
      },
    ];
  }

  function historyChartLabels() {
    return networkHistory.map((day) => formatDateLabel(day.date));
  }

  async function loadSnapshot() {
    snapshot = await siloApi.getAppState();
  }

  async function loadRules() {
    rules = await siloApi.getRules();
  }

  async function loadTodayUsage() {
    const today = new Date().toISOString().slice(0, 10);
    usage = await siloApi.getUsage(today);
  }

  async function loadTimeline() {
    timeline = await siloApi.getUsage90d();
    try {
      heatmapNetworkHistory = await siloApi.getNetworkHistory("90d");
    } catch (e) {
      console.error("Failed to load 90d network history for heatmap:", e);
    }
  }

  async function loadStatsData(range: "today" | RangeKey) {
    try {
      const [uReport, dReport, netHist] = await Promise.all([
        siloApi.getUsageRange(range),
        siloApi.getDataUsage(range),
        siloApi.getNetworkHistory(range === "90d" ? "90d" : range === "7d" ? "7d" : "30d")
      ]);
      statsUsage = uReport;
      statsDataUsage = dReport;
      statsNetworkHistory = netHist;
    } catch (error) {
      console.error("Failed to load statistics range data:", error);
    }
  }

  $effect(() => {
    if (activeView === "stats" && statsRange) {
      void loadStatsData(statsRange);
    }
  });

  async function loadDataUsage(range: RangeKey) {
    dataRange = range;
    dataUsage = await siloApi.getDataUsage(range);
  }

  async function saveRule() {
    if (!ruleDraft.target.trim()) {
      showToast("Rule target is required.", "error");
      return;
    }

    savingRule = true;
    try {
      await siloApi.saveRule({ ...ruleDraft, target: ruleDraft.target.trim() });
      ruleDraft = emptyRule();
      showRuleForm = false;
      await Promise.all([loadRules(), loadSnapshot()]);
      showToast("Rule saved successfully!", "success");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      savingRule = false;
    }
  }

  async function toggleRuleActive(rule: Rule) {
    try {
      const updated = { ...rule, active: !rule.active };
      await siloApi.saveRule(updated);
      await Promise.all([loadRules(), loadSnapshot()]);
      showToast(
        `Rule for ${rule.target} ${updated.active ? "activated" : "paused"}.`,
        "success",
      );
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  function getRuleUsagePercentage(rule: Rule) {
    const remaining = getRuleRemainingSeconds(rule);
    const todayStr = new Date().toISOString().slice(0, 10);
    const limit = rule.extraLimitDate === todayStr
      ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
      : rule.limitSeconds;
    if (limit <= 0) return 100;
    const elapsed = limit - remaining;
    return Math.max(0, Math.min(100, (elapsed / limit) * 100));
  }

  async function removeRule(rule: Rule) {
    if (rule.id === null) return;
    ruleToDelete = rule;
    showConfirmDeleteOverlay = true;
  }

  async function confirmDeleteRule() {
    if (!ruleToDelete || ruleToDelete.id === null) return;
    const id = ruleToDelete.id;
    const target = ruleToDelete.target;
    showConfirmDeleteOverlay = false;
    ruleToDelete = null;
    try {
      await siloApi.deleteRule(id);
      await Promise.all([loadRules(), loadSnapshot()]);
      showToast(`Rule for ${target} deleted.`, "info");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  function cancelDeleteRule() {
    showConfirmDeleteOverlay = false;
    ruleToDelete = null;
  }

  function editRule(rule: Rule) {
    ruleDraft = { ...rule };
    showRuleForm = true;
    activeView = "rules";
  }

  async function toggleFocus() {
    try {
      const enabled = await siloApi.toggleFocusMode();
      if (snapshot) snapshot = { ...snapshot, focusMode: enabled };
      if (boot) boot = { ...boot, focusMode: enabled };
      showToast(
        enabled ? "Focus Mode started!" : "Focus Mode stopped.",
        "info",
      );
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  async function saveSettings() {
    if (!settings) return;
    savingSettings = true;
    try {
      settings = await siloApi.saveSettings(settings);
      if (boot) boot = { ...boot, settings };
      showToast("Settings saved successfully!", "success");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      savingSettings = false;
    }
  }

  async function completeBackup() {
    try {
      settings = await siloApi.markBackupComplete();
      showToast("Backup status updated.", "success");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  async function exportUsage() {
    exporting = true;
    try {
      const result = await siloApi.exportUsageData(dataRange);
      exportPath = result.filePath;
      showToast("Usage data exported successfully!", "success");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      exporting = false;
    }
  }

  async function exportLogs() {
    exporting = true;
    try {
      const result = await siloApi.exportLogs(dataRange);
      exportPath = result.filePath;
      showToast("Logs exported successfully!", "success");
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      exporting = false;
    }
  }

  function formatDuration(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m`;
    return `${safeSeconds}s`;
  }

  function cleanDomain(input: string): string {
    if (!input) return "";
    let cleaned = input.trim().toLowerCase();
    
    // Strip protocol
    if (cleaned.startsWith("https://")) {
      cleaned = cleaned.slice(8);
    } else if (cleaned.startsWith("http://")) {
      cleaned = cleaned.slice(7);
    }
    
    // Strip path
    const slashIdx = cleaned.indexOf("/");
    if (slashIdx !== -1) {
      cleaned = cleaned.slice(0, slashIdx);
    }
    
    // Strip port
    const colonIdx = cleaned.indexOf(":");
    if (colonIdx !== -1) {
      cleaned = cleaned.slice(0, colonIdx);
    }
    
    // Strip www.
    if (cleaned.startsWith("www.")) {
      cleaned = cleaned.slice(4);
    }

    // Keywords mapping
    if (cleaned.includes("youtube")) {
      return "youtube.com";
    }
    if (cleaned.includes("github")) {
      return "github.com";
    }
    if (cleaned.includes("google search") || cleaned === "google") {
      return "google.com";
    }
    if (cleaned.includes("gmail")) {
      return "gmail.com";
    }
    if (cleaned.includes("facebook")) {
      return "facebook.com";
    }
    if (cleaned.includes("twitter") || cleaned === "x") {
      return "x.com";
    }
    if (cleaned.includes("reddit")) {
      return "reddit.com";
    }
    if (cleaned.includes("netflix")) {
      return "netflix.com";
    }
    if (cleaned.includes("linkedin")) {
      return "linkedin.com";
    }
    if (cleaned.includes("stackoverflow")) {
      return "stackoverflow.com";
    }
    if (cleaned.includes("wikipedia")) {
      return "wikipedia.org";
    }
    if (cleaned.includes("amazon")) {
      return "amazon.com";
    }

    // Filter characters
    const filtered = cleaned.replace(/[^a-z0-9.-]/g, "");
    if (filtered.includes(".")) {
      return filtered;
    } else if (filtered.length > 0) {
      return filtered + ".com";
    }
    return "unknown.com";
  }

  function getRuleRemainingSeconds(rule: Rule) {
    const todayStr = new Date().toISOString().slice(0, 10);
    const limit = rule.extraLimitDate === todayStr
      ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
      : rule.limitSeconds;

    let elapsed = 0;
    if (rule.ruleType === "app") {
      const match = usage?.apps.find(
        (a) => a.name.toLowerCase() === rule.target.toLowerCase(),
      );
      if (match) {
        elapsed = match.seconds;
      }
      if (
        snapshot?.activeApp &&
        snapshot.activeApp.app.toLowerCase() === rule.target.toLowerCase()
      ) {
        elapsed += snapshot.activeApp.elapsedSeconds;
      }
    } else if (rule.ruleType === "site") {
      const match = usage?.sites?.find(
        (s) => cleanDomain(s.name) === cleanDomain(rule.target),
      );
      if (match) {
        elapsed = match.seconds;
      }
      if (
        snapshot?.activeApp?.site &&
        cleanDomain(snapshot.activeApp.site) === cleanDomain(rule.target)
      ) {
        elapsed += snapshot.activeApp.elapsedSeconds;
      }
    }

    return Math.max(0, limit - elapsed);
  }

  function getTargetRule(target: string, type: "apps" | "sites") {
    const mappedType = type === "apps" ? "app" : "site";
    return rules.find(
      (r) =>
        r.active &&
        r.ruleType === mappedType &&
        (mappedType === "site"
          ? cleanDomain(r.target) === cleanDomain(target)
          : r.target.toLowerCase() === target.toLowerCase()),
    );
  }

  async function handleRefresh() {
    if (isRefreshing) return;
    isRefreshing = true;
    try {
      await Promise.all([
        loadTodayUsage(),
        loadSnapshot(),
        new Promise((resolve) => setTimeout(resolve, 750)),
      ]);
      refreshKey += 1;
    } catch (err) {
      showToast(toErrorMessage(err), "error");
    } finally {
      isRefreshing = false;
    }
  }

  function formatClock(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    const secs = Math.floor(safeSeconds % 60);
    if (hours > 0)
      return `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
    return `${minutes}:${String(secs).padStart(2, "0")}`;
  }

  async function addTime(ruleId: number, minutes: number) {
    try {
      await siloApi.addRuleTime(ruleId, minutes * 60);
      showViolationOverlay = false;
      violationData = null;
      showWarningOverlay = false;
      warningData = null;
      showCountdownOverlay = false;
      countdownData = null;
      showToast(`Added ${minutes} minutes for today!`, "success");
      await loadAll();
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  function formatBytes(bytes: number) {
    if (bytes >= 1_073_741_824)
      return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${bytes} B`;
  }

  function formatBps(bytes: number) {
    return `${formatBytes(bytes)}/s`;
  }

  function formatDateLabel(date: string) {
    return new Intl.DateTimeFormat(undefined, { weekday: "short" }).format(
      new Date(`${date}T00:00:00`),
    );
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function filteredRules() {
    const query = ruleSearch.trim().toLowerCase();
    if (!query) return rules;
    return rules.filter(
      (rule) =>
        rule.target.toLowerCase().includes(query) ||
        rule.ruleType.includes(query),
    );
  }

  function weekDays() {
    return timeline?.days.slice(-7) ?? [];
  }

  function chartLabels() {
    return weekDays().map((day) => formatDateLabel(day.date));
  }

  function focusChartData() {
    return [
      {
        label: "Tracked time",
        data: weekDays().map((day) => Math.round(day.totalSeconds / 60)),
        backgroundColor: "rgba(20, 184, 166, 0.85)",
        borderColor: "#2dd4bf",
        borderRadius: 6,
      },
    ];
  }

  function networkChartData() {
    const appsTotal = dataUsage?.totalDownloadBytes ?? 0;
    const uploadTotal = dataUsage?.totalUploadBytes ?? 0;
    return [
      {
        label: "Download",
        data: [Math.round(appsTotal / 1_048_576)],
        backgroundColor: "rgba(20, 184, 166, 0.85)",
        borderColor: "#2dd4bf",
        borderRadius: 6,
      },
      {
        label: "Upload",
        data: [Math.round(uploadTotal / 1_048_576)],
        backgroundColor: "rgba(139, 92, 246, 0.85)",
        borderColor: "#a78bfa",
        borderRadius: 6,
      },
    ];
  }

  function totalTrackedSeconds() {
    return timeline?.days.reduce((sum, day) => sum + day.totalSeconds, 0) ?? 0;
  }

  function averageTrackedSeconds() {
    const days = timeline?.days.filter((day) => day.totalSeconds > 0) ?? [];
    if (!days.length) return 0;
    return Math.round(
      days.reduce((sum, day) => sum + day.totalSeconds, 0) / days.length,
    );
  }

  function bestTrackedDay() {
    const days = timeline?.days ?? [];
    return days.reduce(
      (best, day) => (day.totalSeconds > best.totalSeconds ? day : best),
      { date: "", totalSeconds: 0 },
    );
  }

  function totalDataBytes() {
    return (
      (dataUsage?.totalDownloadBytes ?? 0) + (dataUsage?.totalUploadBytes ?? 0)
    );
  }

  function consumerTotal(consumer: DataConsumer) {
    return consumer.downloadBytes + consumer.uploadBytes;
  }

  function getNetworkHeatmapColor(bytes: number) {
    if (bytes > 1_073_741_824) return "bg-violet-300 shadow-[0_0_6px_rgba(167,139,250,0.4)]";
    if (bytes > 104_857_600) return "bg-violet-500";
    if (bytes > 1_048_576) return "bg-violet-900";
    return "bg-slate-900";
  }

  function handleKeyDown(e: KeyboardEvent) {
    const activeEl = document.activeElement;
    if (
      activeEl &&
      (activeEl.tagName === "INPUT" ||
        activeEl.tagName === "TEXTAREA" ||
        activeEl.tagName === "SELECT" ||
        activeEl.getAttribute("contenteditable") === "true")
    ) {
      if (e.key === "Escape") {
        (activeEl as HTMLElement).blur();
        showAppDropdown = false;
        showSiteDropdown = false;
      }
      return;
    }

    if (e.key === "Escape") {
      if (showRuleForm) {
        showRuleForm = false;
        ruleDraft = emptyRule();
        e.preventDefault();
      }
      if (showConfirmDeleteOverlay) {
        showConfirmDeleteOverlay = false;
        ruleToDelete = null;
        e.preventDefault();
      }
      if (showWarningOverlay) {
        showWarningOverlay = false;
        warningData = null;
        e.preventDefault();
      }
      if (showViolationOverlay) {
        showViolationOverlay = false;
        violationData = null;
        e.preventDefault();
      }
      return;
    }

    if (settings?.shortcutsEnabled) {
      switch (e.key.toLowerCase()) {
        case "d":
        case "1":
          activeView = "dashboard";
          e.preventDefault();
          break;
        case "r":
        case "2":
          activeView = "rules";
          e.preventDefault();
          break;
        case "s":
        case "3":
          activeView = "stats";
          e.preventDefault();
          break;
        case "n":
        case "4":
          activeView = "network";
          e.preventDefault();
          break;
        case "p":
        case "5":
        case ",":
          activeView = "settings";
          e.preventDefault();
          break;
        case " ":
          void toggleFocus();
          e.preventDefault();
          break;
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<svelte:head>
  <title>SILO</title>
</svelte:head>

<main
  class="min-h-screen bg-silo text-slate-100"
  ontouchstart={handleTouchStart}
  ontouchend={handleTouchEnd}
  onwheel={handleWheel}
>
  <div
    class="mx-auto min-h-screen w-full max-w-7xl px-4 pb-28 pt-8 sm:px-6 lg:px-8"
  >


    {#if loading}
      <section class="flex min-h-[70vh] items-center justify-center">
        <div class="text-center">
          <div
            class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-teal-400/15 text-teal-300"
          >
            <Database size={26} />
          </div>
          <p class="mt-4 text-lg font-bold">Loading SILO workspace</p>
          <p class="mt-1 text-sm text-slate-500">
            Connecting to local monitoring services.
          </p>
        </div>
      </section>
    {:else if activeView === "dashboard"}
      <section class="space-y-6" transition:fade={{ duration: 150 }}>
        <!-- Premium Header Area -->
        <header class="relative overflow-hidden rounded-3xl border transition-all duration-500 backdrop-blur-md p-8 shadow-2xl
          {snapshot?.focusMode 
            ? 'border-red-500/15 bg-slate-950/60 shadow-red-950/5' 
            : 'border-slate-800/80 bg-slate-950/45 shadow-black/35'}">
          <!-- Pulsing backdrop glow -->
          <div class="absolute -right-16 -top-16 h-48 w-48 rounded-full transition-colors duration-700 blur-3xl animate-pulse-glow
            {snapshot?.focusMode ? 'bg-red-500/5' : 'bg-teal-500/10'}"></div>
          <div class="absolute -left-16 -bottom-16 h-48 w-48 rounded-full transition-colors duration-700 blur-3xl animate-pulse-glow
            {snapshot?.focusMode ? 'bg-rose-500/5' : 'bg-purple-500/10'}"></div>
          
          <!-- Branding + Time Row -->
          <div class="flex flex-col sm:flex-row items-center sm:justify-between gap-4 border-b border-slate-800/40 pb-6 mb-6">
            <div class="text-center sm:text-left">
              <p class="text-[10px] font-black text-teal-400 uppercase tracking-widest mb-1.5">{currentDateStr}</p>
              <h1 class="text-3xl font-black text-slate-100 flex items-center justify-center sm:justify-start gap-2">
                <span>{greetingText}</span>
              </h1>
            </div>
            <div class="flex items-center gap-2.5 bg-slate-900/60 px-4 py-2 rounded-2xl border border-slate-800/50">
              <span class="text-xl font-extrabold bg-gradient-to-r from-teal-300 via-blue-400 to-violet-400 bg-clip-text text-transparent tracking-wider">SILO</span>
              <span class="h-4 w-px bg-slate-800"></span>
              <span class="text-[10px] font-bold text-slate-500 uppercase tracking-wider">v{boot?.version || "0.4.0"}</span>
            </div>
          </div>

          <p class="text-slate-400 text-sm font-semibold tracking-wide">
            {snapshot?.focusMode 
              ? "Shield is actively intercepting notifications and monitoring distractors."
              : "Shield is currently offline. Your limits and blocks are paused."}
          </p>
          
          <div class="mt-6 flex justify-center">
            <div class="relative">
              {#if snapshot?.focusMode}
                <!-- Glowing ring behind button -->
                <div class="absolute -inset-1 rounded-2xl bg-gradient-to-r from-red-500 to-rose-600 opacity-30 blur-md animate-pulse"></div>
              {/if}
              <button 
                class="relative group px-8 py-3.5 rounded-2xl font-extrabold text-sm transition-all duration-300 overflow-hidden flex items-center gap-2.5 shadow-lg
                  {snapshot?.focusMode 
                    ? 'bg-gradient-to-r from-red-500 to-rose-600 text-white shadow-red-500/15 hover:shadow-red-500/25 hover:scale-[1.03] active:scale-[0.98]' 
                    : 'bg-gradient-to-r from-teal-400 to-emerald-500 text-slate-950 shadow-teal-500/15 hover:shadow-teal-500/25 hover:scale-[1.03] active:scale-[0.98]'}" 
                type="button" 
                onclick={toggleFocus}
              >
                <Power size={18} class={snapshot?.focusMode ? 'animate-pulse' : ''} />
                <span>{snapshot?.focusMode ? "Disable Focus Guard" : "Activate Focus Shield"}</span>
              </button>
            </div>
          </div>
        </header>

        <!-- Metrics cards row -->
        <div class="grid gap-5 md:grid-cols-3">
          <!-- Active application -->
          <section class="silo-card p-6 flex flex-col justify-between relative overflow-hidden group hover:border-teal-500/30 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-teal-500/5 blur-xl group-hover:bg-teal-500/10 transition-colors"></div>
            <div>
              <div class="flex items-center justify-between">
                <IconBadge
                  icon={snapshot?.activeApp.site ? Globe : Monitor}
                  tone={snapshot?.activeApp.site ? "teal" : "purple"}
                  label="Active application"
                />
                {#if snapshot?.activeApp.app}
                  <span class="flex items-center gap-1.5 text-xs text-teal-400 font-bold bg-teal-950/40 border border-teal-500/20 px-2 py-0.5 rounded-full">
                    <span class="h-1.5 w-1.5 rounded-full bg-teal-400 animate-radar"></span>
                    Live
                  </span>
                {/if}
              </div>
              
              {#if snapshot?.activeApp.site}
                <p class="mt-6 truncate text-2xl font-black text-slate-100 group-hover:text-teal-300 transition-colors" title={snapshot.activeApp.site}>
                  {snapshot.activeApp.site}
                </p>
                <p class="mt-1 truncate text-xs text-slate-500 font-semibold uppercase tracking-wider">
                  via {snapshot.activeApp.app}
                </p>
              {:else}
                <p class="mt-6 truncate text-2xl font-black text-slate-100 group-hover:text-purple-300 transition-colors" title={snapshot?.activeApp.app || "No active app"}>
                  {snapshot?.activeApp.app || "No active app"}
                </p>
                <p class="mt-1 truncate text-xs text-slate-500 font-semibold uppercase tracking-wider">
                  {snapshot?.activeApp.title || "Current window"}
                </p>
              {/if}
            </div>
            <p
              class="mt-5 flex items-center gap-2 font-mono text-lg font-bold text-teal-300"
            >
              <Clock size={18} />
              {formatClock(snapshot?.activeApp.elapsedSeconds ?? 0)}
            </p>
          </section>

          <!-- Session progress -->
          <section class="silo-card p-6 flex flex-col justify-between relative overflow-hidden group hover:border-purple-500/30 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-purple-500/5 blur-xl group-hover:bg-purple-500/10 transition-colors"></div>
            <div>
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <IconBadge icon={Timer} tone="purple" label="Session progress" />
                  <h2 class="text-sm font-bold text-slate-400">Session Progress</h2>
                </div>
                <span class="text-xs font-mono font-bold text-purple-300">
                  {Math.round(Math.min(100, ((snapshot?.todaySeconds ?? 0) / 28800) * 100))}%
                </span>
              </div>
              <p class="mt-6 text-3xl font-black text-slate-100 font-mono">
                {formatDuration(snapshot?.todaySeconds ?? 0)}
              </p>
              <p class="mt-1 text-xs text-slate-500 font-medium">Tracked focus today (Goal: 8h)</p>
            </div>
            <div class="mt-5">
              <div class="w-full bg-slate-950 h-2.5 rounded-full overflow-hidden relative border border-slate-800/80">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-teal-400 to-purple-500 animate-shimmer"
                  style={`width: ${Math.min(100, ((snapshot?.todaySeconds ?? 0) / 28800) * 100)}%; background-size: 200% 100%;`}
                ></div>
              </div>
              <p class="mt-3 text-[10px] text-slate-500 font-bold uppercase tracking-widest flex items-center gap-1.5">
                <span class="h-1.5 w-1.5 rounded-full {snapshot?.focusMode ? 'bg-emerald-400 animate-pulse' : 'bg-slate-600'}"></span>
                {snapshot?.focusMode ? "Monitoring Active" : "Shield Suspended"}
              </p>
            </div>
          </section>

          <!-- Live focus score -->
          <section class="silo-card p-6 flex flex-col justify-between items-center text-center relative overflow-hidden group hover:border-amber-500/30 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-amber-500/5 blur-xl group-hover:bg-amber-500/10 transition-colors"></div>
            <div class="flex items-center gap-2 self-start">
              <IconBadge icon={Target} tone="yellow" label="Focus score" />
              <h2 class="text-sm font-bold text-slate-400">Live Focus Score</h2>
            </div>
            
            <div class="my-3 flex flex-col items-center">
              <div class="relative flex items-center justify-center">
                <!-- Circular Progress Ring with Gradient -->
                <svg class="h-28 w-28 -rotate-90">
                  <defs>
                    <linearGradient id="focusScoreGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stop-color={focusScoreColor.primary} />
                      <stop offset="100%" stop-color={focusScoreColor.secondary} />
                    </linearGradient>
                  </defs>
                  <circle class="text-slate-900" stroke-width="8" stroke="currentColor" fill="transparent" r="42" cx="56" cy="56" />
                  <circle stroke="url(#focusScoreGradient)" stroke-width="8" stroke-dasharray={2 * Math.PI * 42} stroke-dashoffset={2 * Math.PI * 42 * (1 - focusScoreValue / 100)} stroke-linecap="round" fill="transparent" r="42" cx="56" cy="56" class="transition-all duration-500" style={`filter: drop-shadow(0 0 6px ${focusScoreColor.shadow})`} />
                </svg>
                <div class="absolute text-3xl font-black text-slate-100 font-mono">
                  {focusScoreValue}%
                </div>
              </div>
              <span class="text-[10px] font-black tracking-widest uppercase mt-3 px-3 py-1 rounded-full border transition-colors duration-500 {focusScoreColor.label}">
                {focusScoreLabel}
              </span>
            </div>
            
            <p class="text-[10px] text-slate-500 text-center font-medium">
              Updates dynamically based on focus rules and limit breaches.
            </p>
          </section>
        </div>

        <!-- Weekly pattern and live speeds -->
        <div class="grid gap-5 xl:grid-cols-3">
          <!-- Weekly trend chart -->
          <section class="silo-card p-6 xl:col-span-2 relative overflow-hidden group hover:border-teal-500/20 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-teal-500/5 blur-xl group-hover:bg-teal-500/10 transition-colors"></div>
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-3">
                <IconBadge icon={Activity} tone="teal" label="Focus pattern" />
                <div>
                  <h2 class="text-lg font-bold">Focus Trend</h2>
                  <p class="text-sm text-slate-500">Weekly tracked screen time (minutes)</p>
                </div>
              </div>
            </div>
            {#if weekDays().length}
              <div class="mt-6">
                <TrendChart
                  labels={chartLabels()}
                  datasets={focusChartData()}
                  height={250}
                />
              </div>
            {:else}
              <EmptyState
                icon={ChartColumn}
                title="No trend data yet"
                message="Session history will appear here after SILO records foreground app activity."
              />
            {/if}
          </section>

          <!-- Live Network attribution speed card -->
          <section class="silo-card p-6 flex flex-col justify-between relative overflow-hidden group hover:border-teal-500/30 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-teal-500/5 blur-xl group-hover:bg-teal-500/10 transition-colors"></div>
            <div>
              <div class="flex items-center justify-between gap-3 border-b border-slate-800/60 pb-3">
                <div class="flex items-center gap-3">
                  <IconBadge icon={Wifi} tone="teal" label="Network Speed" />
                  <h2 class="text-lg font-bold">Network Speed</h2>
                </div>
                <span class="flex items-center gap-1 text-[10px] text-emerald-300 font-black px-2.5 py-0.5 rounded-full border border-emerald-500/20 bg-emerald-500/10 animate-pulse">
                  <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
                  LIVE
                </span>
              </div>
              
              <div class="mt-6 space-y-5">
                <div>
                  <div class="flex items-center justify-between text-xs font-semibold text-slate-400 mb-1.5">
                    <span class="flex items-center gap-1.5"><Download size={13} class="text-teal-400" /> Download</span>
                    <span class="font-bold text-slate-200 font-mono">{formatBps(snapshot?.networkSpeed.downloadBps ?? 0)}</span>
                  </div>
                  <div class="w-full bg-slate-950 h-2 rounded-full overflow-hidden relative border border-slate-900">
                    <div 
                      class="h-full bg-teal-400 transition-all duration-300 rounded-full {snapshot?.networkSpeed.downloadBps && snapshot.networkSpeed.downloadBps > 1024 ? 'animate-shimmer bg-gradient-to-r from-teal-400 via-teal-300 to-teal-400' : ''}"
                      style={`width: ${Math.max(4, Math.min(100, ((snapshot?.networkSpeed.downloadBps ?? 0) / 10_485_760) * 100))}%`}
                    ></div>
                  </div>
                </div>
                
                <div>
                  <div class="flex items-center justify-between text-xs font-semibold text-slate-400 mb-1.5">
                    <span class="flex items-center gap-1.5"><Upload size={13} class="text-violet-400" /> Upload</span>
                    <span class="font-bold text-slate-200 font-mono">{formatBps(snapshot?.networkSpeed.uploadBps ?? 0)}</span>
                  </div>
                  <div class="w-full bg-slate-950 h-2 rounded-full overflow-hidden relative border border-slate-900">
                    <div 
                      class="h-full bg-violet-400 transition-all duration-300 rounded-full {snapshot?.networkSpeed.uploadBps && snapshot.networkSpeed.uploadBps > 1024 ? 'animate-shimmer bg-gradient-to-r from-violet-400 via-violet-300 to-violet-400' : ''}"
                      style={`width: ${Math.max(4, Math.min(100, ((snapshot?.networkSpeed.uploadBps ?? 0) / 5_242_880) * 100))}%`}
                    ></div>
                  </div>
                </div>
              </div>
            </div>
            
            <p class="mt-6 rounded-xl border border-slate-800/80 bg-slate-950/45 p-3.5 text-[10px] font-medium text-slate-500 leading-relaxed">
              Speed captures total interface activity. App-specific values log asynchronously.
            </p>
          </section>
        </div>

        <!-- AI insights and Today's Usage breakdown -->
        <div class="grid gap-5 xl:grid-cols-[1.35fr_0.65fr]">
          <!-- Mapped AI Insights -->
          <section class="silo-card p-6 relative overflow-hidden group hover:border-purple-500/20 transition-all duration-300">
            <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-purple-500/5 blur-xl group-hover:bg-purple-500/10 transition-colors"></div>
            <div class="flex items-center gap-3 border-b border-slate-800/60 pb-4 mb-4">
              <IconBadge icon={Sparkles} tone="purple" label="AI insights" />
              <div>
                <h2 class="text-lg font-bold">AI Focus Analysis</h2>
                <p class="text-sm text-slate-500">Real-time productivity suggestions</p>
              </div>
            </div>
            
            <div class="space-y-3.5 mt-4">
              {#if aiInsightsList.length}
                {#each aiInsightsList as insight}
                  <div class="p-3.5 rounded-xl border flex gap-3 text-sm font-medium transition-all duration-200 bg-slate-950/25 hover:bg-slate-950/50 hover:scale-[1.01]
                    {insight.type === 'success'
                      ? 'border-emerald-500/10 hover:border-emerald-500/25 text-emerald-300'
                      : insight.type === 'warn'
                        ? 'border-amber-500/10 hover:border-amber-500/25 text-amber-300'
                        : 'border-blue-500/10 hover:border-blue-500/25 text-blue-300'}">
                    <div class="flex items-start mt-0.5">
                      <span class="h-2 w-2 rounded-full mt-1.5 animate-pulse
                        {insight.type === 'success'
                          ? 'bg-emerald-400'
                          : insight.type === 'warn'
                            ? 'bg-amber-400'
                            : 'bg-blue-400'}"></span>
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="font-bold text-slate-200 mb-0.5 truncate">{insight.title}</p>
                      <p class="text-xs text-slate-400 font-semibold leading-relaxed">{insight.desc}</p>
                    </div>
                  </div>
                {/each}
              {:else}
                <div class="py-10 text-center text-slate-500 text-sm font-semibold flex flex-col items-center justify-center gap-2">
                  <div class="h-5 w-5 border-2 border-teal-500 border-t-transparent rounded-full animate-spin"></div>
                  <span>Analyzing focus logs to generate suggestions...</span>
                </div>
              {/if}
            </div>
          </section>

          <!-- Sleek Usage List -->
          <section class="silo-card p-6 hover:border-teal-500/20 transition-all duration-300 flex flex-col justify-between">
            <div>
              <div class="flex items-center justify-between gap-3">
                <div class="flex items-center gap-3">
                  <IconBadge icon={ChartColumn} tone="teal" label="Usage" />
                  <h2 class="text-lg font-bold">Today's Usage</h2>
                </div>
                <button
                  class="silo-icon-button p-1 text-slate-400 hover:text-teal-300 hover:bg-slate-800 transition rounded-lg"
                  type="button"
                  aria-label="Refresh usage"
                  onclick={handleRefresh}
                  disabled={isRefreshing}
                >
                  <RotateCcw size={15} class={isRefreshing ? "animate-spin text-teal-400" : ""} />
                </button>
              </div>
              
              <div class="mt-4 flex rounded-lg bg-slate-950 p-1">
                <button
                  class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition
                    {usageTab === 'apps' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                  type="button"
                  onclick={() => (usageTab = "apps")}
                >
                  Applications
                </button>
                <button
                  class="flex-1 rounded-md py-1.5 text-center text-xs font-semibold transition
                    {usageTab === 'sites' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                  type="button"
                  onclick={() => (usageTab = "sites")}
                >
                  Websites
                </button>
              </div>

              {#key refreshKey}
                <div 
                  in:scale={{ duration: 400, start: 0.98, opacity: 0 }}
                  class="mt-5 flex flex-col gap-3"
                >
                  {#if usageTab === "apps"}
                    {#if usage?.apps.length}
                      {#each usage.apps.slice(0, 5) as app}
                        {@const targetRule = getTargetRule(app.name, usageTab)}
                        <div class="silo-card p-3 flex flex-col gap-2.5 bg-slate-900/30 hover:border-slate-700/80 hover:bg-slate-900/50 transition-all duration-200">
                          <div class="flex items-center justify-between gap-4">
                            <!-- Leading: Icon & Name -->
                            <div class="flex items-center gap-3 min-w-0">
                              <IconBadge
                                icon={Monitor}
                                tone="purple"
                                label={app.name}
                                size="md"
                              />
                              <div class="min-w-0">
                                <span class="truncate font-bold text-sm text-slate-200 block" title={app.name}>
                                  {app.name}
                                </span>
                                {#if targetRule}
                                  <span class="text-[10px] text-slate-500 font-bold mt-0.5 block">
                                    Limit: {formatDuration(targetRule.limitSeconds)}
                                  </span>
                                {:else}
                                  <span class="text-[10px] text-slate-500 font-semibold mt-0.5 block">
                                    {((app.seconds / Math.max(1, usage.totalSeconds)) * 100).toFixed(0)}% of today
                                  </span>
                                {/if}
                              </div>
                            </div>

                            <!-- Trailing: Duration & Warning Tier Badge -->
                            <div class="flex items-center gap-3 shrink-0">
                              <span class="text-sm font-black text-slate-100 font-mono">
                                {formatDuration(app.seconds)}
                              </span>
                              
                              {#if targetRule}
                                {@const remainingSecs = getRuleRemainingSeconds(targetRule)}
                                <span class="text-[10px] font-black px-2.5 py-1 rounded-full border 
                                  {remainingSecs <= 0 ? 'bg-red-500/10 text-red-400 border-red-500/20' :
                                   remainingSecs <= 60 ? 'bg-red-500/10 text-red-400 border-red-500/20 animate-pulse' :
                                   remainingSecs <= 300 ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' :
                                   'bg-teal-500/10 text-teal-300 border-teal-500/20'}">
                                  {remainingSecs <= 0 ? 'Blocked' :
                                   remainingSecs <= 60 ? 'Critical' :
                                   remainingSecs <= 300 ? 'Warning' :
                                   'Healthy'}
                                </span>
                              {/if}
                            </div>
                          </div>

                          <!-- Progress Bar of limit or share of total focus -->
                          <div class="w-full bg-slate-950/80 h-1 rounded-full overflow-hidden border border-slate-900">
                            {#if targetRule}
                              {@const remainingSecs = getRuleRemainingSeconds(targetRule)}
                              {@const totalLimit = targetRule.extraLimitDate === new Date().toISOString().slice(0, 10) ? targetRule.limitSeconds + (targetRule.extraLimitSeconds ?? 0) : targetRule.limitSeconds}
                              {@const usagePct = Math.min(100, ((totalLimit - remainingSecs) / Math.max(1, totalLimit)) * 100)}
                              <div 
                                class="h-full rounded-full transition-all duration-500 
                                  {remainingSecs <= 0 ? 'bg-red-500' :
                                   remainingSecs <= 300 ? 'bg-amber-500' :
                                   'bg-teal-500'}"
                                style={`width: ${usagePct}%`}
                              ></div>
                            {:else}
                              <div 
                                class="h-full rounded-full bg-slate-700/60"
                                style={`width: ${Math.min(100, (app.seconds / Math.max(1, usage?.totalSeconds ?? 1)) * 100)}%`}
                              ></div>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    {:else}
                      <EmptyState
                        compact
                        icon={Monitor}
                        title="No usage yet"
                        message="Tracked applications will appear here."
                      />
                    {/if}
                  {:else}
                    {#if usage?.sites?.length}
                      {#each usage.sites.slice(0, 5) as site}
                        {@const targetRule = getTargetRule(site.name, usageTab)}
                        <div class="silo-card p-3 flex flex-col gap-2.5 bg-slate-900/30 hover:border-slate-700/80 hover:bg-slate-900/50 transition-all duration-200">
                          <div class="flex items-center justify-between gap-4">
                            <!-- Leading: Icon & Name -->
                            <div class="flex items-center gap-3 min-w-0">
                              <IconBadge
                                icon={Globe}
                                tone="teal"
                                label={site.name}
                                size="md"
                              />
                              <div class="min-w-0">
                                <span class="truncate font-bold text-sm text-slate-200 block" title={site.name}>
                                  {site.name}
                                </span>
                                {#if targetRule}
                                  <span class="text-[10px] text-slate-500 font-bold mt-0.5 block">
                                    Limit: {formatDuration(targetRule.limitSeconds)}
                                  </span>
                                {:else}
                                  <span class="text-[10px] text-slate-500 font-semibold mt-0.5 block">
                                    {((site.seconds / Math.max(1, usage.totalSeconds)) * 100).toFixed(0)}% of today
                                  </span>
                                {/if}
                              </div>
                            </div>

                            <!-- Trailing: Duration & Warning Tier Badge -->
                            <div class="flex items-center gap-3 shrink-0">
                              <span class="text-sm font-black text-slate-100 font-mono">
                                {formatDuration(site.seconds)}
                              </span>
                              
                              {#if targetRule}
                                {@const remainingSecs = getRuleRemainingSeconds(targetRule)}
                                <span class="text-[10px] font-black px-2.5 py-1 rounded-full border 
                                  {remainingSecs <= 0 ? 'bg-red-500/10 text-red-400 border-red-500/20' :
                                   remainingSecs <= 60 ? 'bg-red-500/10 text-red-400 border-red-500/20 animate-pulse' :
                                   remainingSecs <= 300 ? 'bg-amber-500/10 text-amber-400 border-amber-500/20' :
                                   'bg-teal-500/10 text-teal-300 border-teal-500/20'}">
                                  {remainingSecs <= 0 ? 'Blocked' :
                                   remainingSecs <= 60 ? 'Critical' :
                                   remainingSecs <= 300 ? 'Warning' :
                                   'Healthy'}
                                </span>
                              {/if}
                            </div>
                          </div>

                          <!-- Progress Bar of limit or share of total focus -->
                          <div class="w-full bg-slate-950/80 h-1 rounded-full overflow-hidden border border-slate-900">
                            {#if targetRule}
                              {@const remainingSecs = getRuleRemainingSeconds(targetRule)}
                              {@const totalLimit = targetRule.extraLimitDate === new Date().toISOString().slice(0, 10) ? targetRule.limitSeconds + (targetRule.extraLimitSeconds ?? 0) : targetRule.limitSeconds}
                              {@const usagePct = Math.min(100, ((totalLimit - remainingSecs) / Math.max(1, totalLimit)) * 100)}
                              <div 
                                class="h-full rounded-full transition-all duration-500 
                                  {remainingSecs <= 0 ? 'bg-red-500' :
                                   remainingSecs <= 300 ? 'bg-amber-500' :
                                   'bg-teal-500'}"
                                style={`width: ${usagePct}%`}
                              ></div>
                            {:else}
                              <div 
                                class="h-full rounded-full bg-slate-700/60"
                                style={`width: ${Math.min(100, (site.seconds / Math.max(1, usage?.totalSeconds ?? 1)) * 100)}%`}
                              ></div>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    {:else}
                      <EmptyState
                        compact
                        icon={Globe}
                        title="No usage yet"
                        message="Tracked websites will appear here."
                      />
                    {/if}
                  {/if}
                </div>
              {/key}
            </div>
            
            <div
              class="mt-6 border-t border-slate-800 pt-4 text-right text-xl font-black text-teal-300 font-mono"
            >
              {formatDuration(usage?.totalSeconds ?? 0)}
            </div>
          </section>
        </div>
      </section>

    {:else if activeView === "rules"}
      <section class="mx-auto max-w-5xl space-y-6">
        <header
          class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
        >
          <div>
            <h1 class="text-4xl font-black">Rules &amp; Limits</h1>
            <p class="mt-2 text-slate-400">
              Manage your app and website restrictions
            </p>
          </div>
          <button
            class="silo-button flex items-center gap-2"
            type="button"
            onclick={() => {
              if (showRuleForm) {
                showRuleForm = false;
                ruleDraft = emptyRule();
              } else {
                ruleDraft = emptyRule();
                showRuleForm = true;
              }
            }}
          >
            {#if showRuleForm}
              <RotateCcw size={18} />
              Cancel / Close
            {:else}
              <Plus size={18} />
              New Limit Rule
            {/if}
          </button>
        </header>

        {#if showRuleForm}
          <div transition:slide={{ duration: 250 }}>
            <section class="silo-card p-6 border border-teal-500/25 bg-slate-900/60 shadow-xl relative overflow-hidden">
              <div class="absolute top-0 right-0 w-32 h-32 bg-teal-500/5 blur-3xl rounded-full pointer-events-none"></div>
              
              <h2 class="text-lg font-bold mb-4 flex items-center gap-2 text-teal-300">
                <Sparkles size={18} />
                {ruleDraft.id ? 'Modify Limit Rule' : 'Create New Limit Rule'}
              </h2>

              <div class="grid gap-4 lg:grid-cols-[1fr_170px_150px_110px]">
                <label class="text-sm font-semibold text-slate-300 relative">
                  Target (App or Website Domain)
                  {#if ruleDraft.ruleType === "app"}
                    <input
                      class="silo-input mt-2 border-slate-700 focus:border-teal-500 focus:ring-1 focus:ring-teal-500/35 transition"
                      placeholder="chrome.exe"
                      bind:value={ruleDraft.target}
                      onfocus={() => {
                        showAppDropdown = true;
                        void loadAvailableApps();
                      }}
                      onblur={() => (showAppDropdown = false)}
                    />
                    {#if showAppDropdown && filteredAvailableApps().length}
                      <div
                        transition:slide={{ duration: 150 }}
                        class="absolute left-0 right-0 z-50 mt-1 max-h-48 overflow-y-auto rounded-lg border border-slate-700 bg-slate-950 shadow-xl scrollbar-thin scrollbar-thumb-slate-700"
                      >
                        {#each filteredAvailableApps() as app}
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-left text-sm hover:bg-teal-500/20 hover:text-teal-300 transition-colors"
                            onmousedown={(e) => {
                              e.preventDefault();
                              ruleDraft.target = app;
                              showAppDropdown = false;
                            }}
                          >
                            {app}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  {:else}
                    <input
                      class="silo-input mt-2 border-slate-700 focus:border-teal-500 focus:ring-1 focus:ring-teal-500/35 transition"
                      placeholder="youtube.com"
                      bind:value={ruleDraft.target}
                      onfocus={() => (showSiteDropdown = true)}
                      onblur={() => (showSiteDropdown = false)}
                    />
                    {#if showSiteDropdown && filteredAvailableSites().length}
                      <div
                        transition:slide={{ duration: 150 }}
                        class="absolute left-0 right-0 z-50 mt-1 max-h-48 overflow-y-auto rounded-lg border border-slate-700 bg-slate-950 shadow-xl scrollbar-thin scrollbar-thumb-slate-700"
                      >
                        {#each filteredAvailableSites() as site}
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-left text-sm hover:bg-teal-500/20 hover:text-teal-300 transition-colors"
                            onmousedown={(e) => {
                              e.preventDefault();
                              ruleDraft.target = site;
                              showSiteDropdown = false;
                            }}
                          >
                            {site}
                          </button>
                        {/each}
                      </div>
                    {/if}
                  {/if}
                </label>
                <label class="text-sm font-semibold text-slate-300">
                  Type
                  <select class="silo-input mt-2 border-slate-700" bind:value={ruleDraft.ruleType}>
                    <option value="app">App</option>
                    <option value="site">Site</option>
                  </select>
                </label>
                <label class="text-sm font-semibold text-slate-300">
                  Enforcement
                  <select
                    class="silo-input mt-2 border-slate-700"
                    bind:value={ruleDraft.enforcement}
                  >
                    <option value="soft">Soft Warning</option>
                    <option value="hard">Hard Block</option>
                    <option value="warn">Warning</option>
                  </select>
                </label>
                <label class="text-sm font-semibold text-slate-300">
                  Minutes
                  <input
                    class="silo-input mt-2 border-slate-700"
                    min="0"
                    type="number"
                    value={Math.round(ruleDraft.limitSeconds / 60)}
                    oninput={(event) =>
                      (ruleDraft.limitSeconds =
                        Number((event.currentTarget as HTMLInputElement).value) *
                        60)}
                  />
                </label>
              </div>
              <div class="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-slate-800/80 pt-4">
                <label
                  class="inline-flex items-center gap-3 text-sm font-semibold text-slate-300"
                >
                  <input
                    class="h-4 w-4 rounded border-slate-600 bg-slate-950 accent-teal-400"
                    type="checkbox"
                    bind:checked={ruleDraft.active}
                  />
                  Active rule immediately
                </label>
                <div class="flex gap-2">
                  <button
                    class="silo-button-secondary bg-slate-800 hover:bg-slate-700"
                    type="button"
                    onclick={() => {
                      ruleDraft = emptyRule();
                      showRuleForm = false;
                    }}>Clear &amp; Close</button
                  >
                  <button
                    class="silo-button"
                    type="button"
                    onclick={saveRule}
                    disabled={savingRule}
                  >
                    {savingRule
                      ? "Saving"
                      : ruleDraft.id
                        ? "Save Rule"
                        : "Add Rule"}
                  </button>
                </div>
              </div>
            </section>
          </div>
        {/if}

        <div class="relative max-w-md">
          <Search
            class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500"
            size={17}
          />
          <input
            class="silo-input pl-10"
            placeholder="Filter rules..."
            bind:value={ruleSearch}
          />
        </div>

        <section class="space-y-4">
          {#if filteredRules().length}
            {#each filteredRules() as rule (rule.id ?? rule.target)}
              <article 
                class="silo-card p-6 flex flex-col gap-4 border border-slate-800 hover:border-slate-700 bg-slate-900/40 hover:bg-slate-900/60 transition-all duration-300 relative group overflow-hidden"
                transition:slide={{ duration: 200 }}
              >
                {#if rule.active}
                  <div class="absolute top-0 left-0 w-[4px] h-full bg-gradient-to-b from-teal-400 to-teal-600"></div>
                {:else}
                  <div class="absolute top-0 left-0 w-[4px] h-full bg-slate-700"></div>
                {/if}

                <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
                  <div class="flex items-center gap-4 min-w-0">
                    <IconBadge
                      icon={rule.ruleType === "site" ? Globe : Monitor}
                      tone={rule.ruleType === "site" ? "teal" : "purple"}
                      label={rule.ruleType}
                      size="lg"
                    />
                    <div class="min-w-0">
                      <h2 class="truncate text-xl font-black text-slate-100 flex items-center gap-2.5">
                        {rule.target}
                        <span class="text-[10px] uppercase font-bold tracking-widest px-2 py-0.5 rounded-full border border-slate-800 bg-slate-950 text-slate-400">
                          {rule.ruleType}
                        </span>
                      </h2>
                      
                      <div class="mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1.5 text-xs text-slate-400 font-semibold">
                        <span class="flex items-center gap-1.5">
                          <Clock size={13} class="text-slate-500" />
                          Limit: {formatDuration(rule.limitSeconds)} daily
                        </span>
                        <span class="flex items-center gap-1.5 text-teal-400">
                          <Timer size={13} class="text-teal-400" />
                          Remaining: {formatDuration(getRuleRemainingSeconds(rule))}
                        </span>
                      </div>
                    </div>
                  </div>

                  <!-- Toggle & Action Buttons -->
                  <div class="flex items-center justify-between gap-4 md:justify-end md:shrink-0 mt-2 md:mt-0">
                    <!-- Active Switch -->
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold uppercase tracking-wider {rule.active ? 'text-teal-400' : 'text-slate-500'}">
                        {rule.active ? 'Active' : 'Paused'}
                      </span>
                      <ToggleSwitch checked={rule.active} label="Active state" onchange={() => toggleRuleActive(rule)} />
                    </div>

                    <!-- Actions Divider -->
                    <div class="hidden md:block h-6 w-[1px] bg-slate-800"></div>

                    <!-- Action Buttons -->
                    <div class="flex items-center gap-1">
                      <button
                        class="silo-icon-button p-2 text-slate-400 hover:text-slate-200 hover:bg-slate-800/80 rounded-lg transition"
                        type="button"
                        aria-label={`Edit ${rule.target}`}
                        onclick={() => editRule(rule)}
                      >
                        <Info size={16} />
                      </button>
                      <button
                        class="silo-icon-button p-2 text-red-400 hover:text-red-300 hover:bg-red-950/45 rounded-lg transition"
                        type="button"
                        aria-label={`Delete ${rule.target}`}
                        onclick={() => removeRule(rule)}
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </div>
                </div>

                <!-- Progress Bar & Enforcement Badge -->
                <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between mt-2 pt-2 border-t border-slate-800/40">
                  <!-- Progress slider -->
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between text-xs font-semibold mb-1 text-slate-400">
                      <span>Usage Limit</span>
                      <span>{Math.round(getRuleUsagePercentage(rule))}% used</span>
                    </div>
                    <div class="w-full bg-slate-950 h-2 rounded-full overflow-hidden border border-slate-900">
                      <div 
                        class="h-full rounded-full transition-all duration-300
                          {getRuleUsagePercentage(rule) >= 90 
                            ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]' 
                            : getRuleUsagePercentage(rule) >= 70 
                              ? 'bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.5)]' 
                              : 'bg-teal-400 shadow-[0_0_8px_rgba(45,212,191,0.5)]'
                          }" 
                        style={`width: ${getRuleUsagePercentage(rule)}%`}
                      ></div>
                    </div>
                  </div>

                  <!-- Enforcement Badge -->
                  <div class="shrink-0 flex items-center md:pl-6 self-start md:self-auto mt-2 md:mt-0">
                    <span
                      class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-xs font-bold uppercase tracking-wider border
                        {rule.enforcement === 'hard'
                          ? 'bg-red-500/10 border-red-500/20 text-red-300'
                          : rule.enforcement === 'soft'
                            ? 'bg-amber-500/10 border-amber-500/20 text-amber-300'
                            : 'bg-blue-500/10 border-blue-500/20 text-blue-300'}"
                    >
                      <span class="w-1.5 h-1.5 rounded-full 
                        {rule.enforcement === 'hard' 
                          ? 'bg-red-400 animate-pulse' 
                          : rule.enforcement === 'soft' 
                            ? 'bg-amber-400' 
                            : 'bg-blue-400'}"
                      ></span>
                      {rule.enforcement === 'hard'
                        ? 'Hard Block'
                        : rule.enforcement === 'soft'
                          ? 'Soft Warning'
                          : 'Warning'}
                    </span>
                  </div>
                </div>
              </article>
            {/each}
          {:else}
            <EmptyState
              icon={Shield}
              title="No rules found"
              message="Add an app or website rule to start managing distractions."
            />
          {/if}
        </section>
      </section>
    {:else if activeView === "stats"}
      <section class="mx-auto max-w-6xl space-y-6" transition:fade={{ duration: 150 }}>
        <!-- Header & Top bar -->
        <header class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between border-b border-slate-800/60 pb-5">
          <div>
            <h1 class="text-4xl font-black tracking-tight text-slate-100 flex items-center gap-3">
              <ChartColumn class="text-teal-400" size={32} />
              Statistics &amp; Insights
            </h1>
            <p class="mt-1 text-sm text-slate-400">Deep usage insights across applications and sites</p>
          </div>

          <!-- Time Range Selector -->
          <div class="flex rounded-xl bg-slate-900/90 border border-slate-800 p-1 w-fit shadow-lg shadow-black/20">
            {#each [{ key: "today", label: "Today" }, { key: "7d", label: "7 Days" }, { key: "30d", label: "30 Days" }, { key: "90d", label: "90 Days" }] as range}
              <button
                class="rounded-lg px-4 py-2 text-xs font-bold transition-all duration-200 {statsRange === range.key
                  ? 'bg-teal-400 text-slate-950 font-black shadow-md'
                  : 'text-slate-400 hover:text-slate-100'}"
                type="button"
                onclick={() => {
                  statsRange = range.key as RangeKey;
                }}
              >
                {range.label}
              </button>
            {/each}
          </div>
        </header>

        <!-- Sub-navigation Tabs (Screen Time, Network, Habits) -->
        <div class="flex border-b border-slate-800/80">
          <button
            class="px-5 py-3 text-sm font-bold border-b-2 transition-all duration-200 flex items-center gap-2
              {statsSubTab === 'screentime' ? 'border-teal-400 text-teal-300' : 'border-transparent text-slate-400 hover:text-slate-200'}"
            type="button"
            onclick={() => statsSubTab = "screentime"}
          >
            <Clock size={16} /> Screen Time
          </button>
          <button
            class="px-5 py-3 text-sm font-bold border-b-2 transition-all duration-200 flex items-center gap-2
              {statsSubTab === 'network' ? 'border-violet-400 text-violet-300' : 'border-transparent text-slate-400 hover:text-slate-200'}"
            type="button"
            onclick={() => statsSubTab = "network"}
          >
            <Wifi size={16} /> Network Usage
          </button>
          <button
            class="px-5 py-3 text-sm font-bold border-b-2 transition-all duration-200 flex items-center gap-2
              {statsSubTab === 'habits' ? 'border-amber-400 text-amber-300' : 'border-transparent text-slate-400 hover:text-slate-200'}"
            type="button"
            onclick={() => statsSubTab = "habits"}
          >
            <Flame size={16} /> Activity &amp; Habits
          </button>
        </div>

        {#if statsSubTab === "screentime"}
          <!-- Screen Time Insights Subtab -->
          <div class="space-y-6" transition:fade={{ duration: 100 }}>
            <!-- Screen Time Metrics -->
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <MetricCard
                icon={Clock}
                title="Total Screen Time"
                value={formatDuration(statsUsage?.totalSeconds ?? 0)}
                trend={`Total for ${statsRange === 'today' ? 'today' : statsRange}`}
              />
              <MetricCard
                icon={Monitor}
                tone="purple"
                title="Active Applications"
                value={String(statsUsage?.apps.length ?? 0)}
                caption="Attributed local apps"
              />
              <MetricCard
                icon={Globe}
                tone="blue"
                title="Active Websites"
                value={String(statsUsage?.sites?.length ?? 0)}
                caption="Attributed web domains"
              />
              <MetricCard
                icon={Award}
                tone="yellow"
                title="Top Time Sink"
                value={statsScreenTab === 'apps' 
                  ? (statsUsage?.apps[0]?.name || "None") 
                  : (statsUsage?.sites?.[0]?.name || "None")}
                caption={statsScreenTab === 'apps'
                  ? (statsUsage?.apps[0] ? formatDuration(statsUsage.apps[0].seconds) : "No app logged")
                  : (statsUsage?.sites?.[0] ? formatDuration(statsUsage.sites[0].seconds) : "No site logged")}
              />
            </div>

            <!-- Screen Time Grid Breakdown -->
            <section class="silo-card p-6">
              <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-slate-800/60 pb-4">
                <div class="flex items-center gap-3">
                  <IconBadge icon={ChartColumn} tone="teal" label="Focus Breakdown" />
                  <div>
                    <h2 class="text-lg font-bold">Focus Breakdown</h2>
                    <p class="text-sm text-slate-500">Compare application vs browser domain screen time</p>
                  </div>
                </div>

                <!-- Apps/Sites sub-toggle -->
                <div class="flex rounded-lg bg-slate-950 p-1">
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {statsScreenTab === 'apps' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => { statsScreenTab = "apps"; statsSearch = ""; }}
                  >
                    Applications
                  </button>
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {statsScreenTab === 'sites' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => { statsScreenTab = "sites"; statsSearch = ""; }}
                  >
                    Websites
                  </button>
                </div>
              </div>

              <!-- Search filter inside stats list -->
              <div class="mt-5 relative w-full">
                <Search
                  class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500"
                  size={16}
                />
                <input
                  class="silo-input pl-10 w-full"
                  placeholder={`Search ${statsScreenTab === 'apps' ? 'applications' : 'websites'}...`}
                  bind:value={statsSearch}
                />
              </div>

              <!-- List with bars -->
              <div class="mt-5 space-y-3">
                {#if filteredStatsScreenList.length}
                  {#each paginatedStatsScreenList as item, index}
                    {@const percentage = ((item.seconds / Math.max(1, statsUsage?.totalSeconds ?? 0)) * 100).toFixed(0)}
                    {@const isApp = statsScreenTab === "apps"}
                    {@const ruleTarget = isApp ? item.name : cleanDomain(item.name)}
                    {@const targetRule = getTargetRule(ruleTarget, statsScreenTab)}

                    <div class="silo-card p-3.5 flex flex-col gap-3 bg-slate-900/30 hover:border-slate-700/80 transition-colors">
                      <div class="flex items-center justify-between gap-4">
                        <div class="flex items-center gap-3 min-w-0">
                          <IconBadge
                            icon={isApp ? Monitor : Globe}
                            tone={isApp ? "purple" : "teal"}
                            label={item.name}
                            size="md"
                          />
                          <div class="min-w-0">
                            <span class="truncate font-bold text-sm text-slate-200 block" title={item.name}>
                              {item.name}
                            </span>
                            <span class="text-[10px] text-slate-500 font-semibold mt-0.5 block">
                              {percentage}% of range screen time
                            </span>
                          </div>
                        </div>

                        <!-- Time & Actions -->
                        <div class="flex items-center gap-3 shrink-0">
                          <span class="text-sm font-black text-slate-100 font-mono">
                            {formatDuration(item.seconds)}
                          </span>

                          {#if targetRule}
                            <span class="text-[10px] font-black px-2 py-0.5 rounded-full border bg-teal-500/10 text-teal-300 border-teal-500/20">
                              Limited
                            </span>
                          {:else}
                            <!-- Action button to immediately create a rule! -->
                            <button
                              class="silo-icon-button p-1.5 text-slate-500 hover:text-teal-400 hover:bg-slate-800 rounded transition"
                              type="button"
                              title={`Create rule limit for ${item.name}`}
                              onclick={() => {
                                ruleDraft = {
                                  ...emptyRule(),
                                  ruleType: isApp ? "app" : "site",
                                  target: ruleTarget
                                };
                                showRuleForm = true;
                                activeView = "rules";
                              }}
                            >
                              <Plus size={15} />
                            </button>
                          {/if}
                        </div>
                      </div>

                      <!-- Progress Indicator -->
                      <div class="w-full bg-slate-950 h-1.5 rounded-full overflow-hidden">
                        <div 
                          class="h-full rounded-full bg-gradient-to-r {isApp ? 'from-purple-500 to-teal-400' : 'from-teal-500 to-emerald-400'}"
                          style={`width: ${Math.max(4, Math.min(100, (item.seconds / Math.max(1, statsUsage?.totalSeconds ?? 0)) * 100))}%`}
                        ></div>
                      </div>
                    </div>
                  {/each}

                  <!-- Pagination Controller -->
                  {#if totalScreenPages > 1}
                    <div class="flex items-center justify-between border-t border-slate-800/60 pt-4 mt-2">
                      <button
                        class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
                        type="button"
                        disabled={screenTimePage === 1}
                        onclick={() => screenTimePage -= 1}
                      >
                        Previous
                      </button>
                      <span class="text-xs text-slate-400 font-semibold">
                        Page {screenTimePage} of {totalScreenPages}
                      </span>
                      <button
                        class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
                        type="button"
                        disabled={screenTimePage === totalScreenPages}
                        onclick={() => screenTimePage += 1}
                      >
                        Next
                      </button>
                    </div>
                  {/if}
                {:else}
                  <EmptyState
                    compact
                    icon={Search}
                    title="No records found"
                    message="No tracking records found for this period."
                  />
                {/if}
              </div>
            </section>
          </div>

        {:else if statsSubTab === "network"}
          <!-- Network Usage Insights Subtab -->
          <div class="space-y-6" transition:fade={{ duration: 100 }}>
            <!-- Network Metrics -->
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <MetricCard
                icon={Wifi}
                title="Total Network Data"
                value={formatBytes((statsDataUsage?.totalDownloadBytes ?? 0) + (statsDataUsage?.totalUploadBytes ?? 0))}
                trend={`Total for ${statsRange}`}
              />
              <MetricCard
                icon={Download}
                tone="teal"
                title="Download Bytes"
                value={formatBytes(statsDataUsage?.totalDownloadBytes ?? 0)}
                caption="Total data received"
              />
              <MetricCard
                icon={Upload}
                tone="purple"
                title="Upload Bytes"
                value={formatBytes(statsDataUsage?.totalUploadBytes ?? 0)}
                caption="Total data sent"
              />
              <MetricCard
                icon={Award}
                tone="yellow"
                title="Top Network App"
                value={statsNetworkTab === 'apps'
                  ? (statsDataUsage?.apps[0]?.name || "None")
                  : (statsDataUsage?.sites[0]?.name || "None")}
                caption={statsNetworkTab === 'apps'
                  ? (statsDataUsage?.apps[0] ? formatBytes(consumerTotal(statsDataUsage.apps[0])) : "No data")
                  : (statsDataUsage?.sites[0] ? formatBytes(consumerTotal(statsDataUsage.sites[0])) : "No data")}
              />
            </div>

            <!-- Network History Chart -->
            <section class="silo-card p-6">
              <div class="flex items-center justify-between border-b border-slate-800/60 pb-4 mb-5">
                <div>
                  <h2 class="text-lg font-bold">Network Volume Trend</h2>
                  <p class="text-sm text-slate-500">Daily upload and download size trend</p>
                </div>
                <span class="text-xs text-slate-400 px-2.5 py-1 rounded-full border border-slate-800 bg-slate-900/50">
                  History: {statsRange}
                </span>
              </div>
              
              {#if statsNetworkHistory.length}
                <div class="mt-4">
                  <TrendChart
                    labels={statsNetworkHistory.map(day => formatDateLabel(day.date))}
                    datasets={[
                      {
                        label: "Upload (MB)",
                        data: statsNetworkHistory.map(day => Math.round(day.uploadBytes / 1_048_576)),
                        backgroundColor: "rgba(139, 92, 246, 0.85)",
                        borderColor: "#a78bfa",
                        borderRadius: 4,
                      },
                      {
                        label: "Download (MB)",
                        data: statsNetworkHistory.map(day => Math.round(day.downloadBytes / 1_048_576)),
                        backgroundColor: "rgba(20, 184, 166, 0.85)",
                        borderColor: "#2dd4bf",
                        borderRadius: 4,
                      }
                    ]}
                    type="bar"
                    height={250}
                  />
                </div>
              {:else}
                <EmptyState
                  icon={Wifi}
                  title="No trend data available"
                  message="Trend chart will populate once background network activity is logged."
                />
              {/if}
            </section>

            <!-- Network breakdown list -->
            <section class="silo-card p-6">
              <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-slate-800/60 pb-4">
                <div class="flex items-center gap-3">
                  <IconBadge icon={Database} tone="purple" label="Network Consumers" />
                  <div>
                    <h2 class="text-lg font-bold">Attributed Traffic</h2>
                    <p class="text-sm text-slate-500">Bandwidth consumers logged during sample intervals</p>
                  </div>
                </div>

                <!-- subtoggle -->
                <div class="flex rounded-lg bg-slate-950 p-1">
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {statsNetworkTab === 'apps' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => { statsNetworkTab = "apps"; statsSearch = ""; }}
                  >
                    Applications
                  </button>
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {statsNetworkTab === 'sites' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => { statsNetworkTab = "sites"; statsSearch = ""; }}
                  >
                    Websites
                  </button>
                </div>
              </div>

              <!-- Search -->
              <div class="mt-5 relative w-full">
                <Search
                  class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500"
                  size={16}
                />
                <input
                  class="silo-input pl-10 w-full"
                  placeholder={`Search traffic by ${statsNetworkTab === 'apps' ? 'apps' : 'domain'}...`}
                  bind:value={statsSearch}
                />
              </div>

              <!-- List -->
              <div class="mt-5 space-y-3">
                {#if filteredStatsNetworkList.length}
                  {#each paginatedStatsNetworkList as row}
                    {@const totalBytes = consumerTotal(row)}
                    {@const rangeTotal = (statsDataUsage?.totalDownloadBytes ?? 0) + (statsDataUsage?.totalUploadBytes ?? 0)}
                    {@const percentage = ((totalBytes / Math.max(1, rangeTotal)) * 100).toFixed(0)}

                    <div class="silo-card p-3.5 flex flex-col gap-3 bg-slate-900/30 hover:border-slate-700/80 transition-colors">
                      <div class="flex items-center justify-between gap-4">
                        <div class="flex items-center gap-3 min-w-0">
                          <IconBadge
                            icon={statsNetworkTab === 'apps' ? Monitor : Globe}
                            tone="purple"
                            label={row.name}
                            size="md"
                          />
                          <div class="min-w-0">
                            <span class="truncate font-bold text-sm text-slate-200 block" title={row.name}>
                              {row.name}
                            </span>
                            <span class="text-[10px] text-slate-500 font-semibold mt-0.5 block">
                              Down: {formatBytes(row.downloadBytes)} · Up: {formatBytes(row.uploadBytes)}
                            </span>
                          </div>
                        </div>
                        <div class="text-right shrink-0">
                          <span class="text-sm font-black text-slate-100 font-mono block">
                            {formatBytes(totalBytes)}
                          </span>
                          <span class="text-[10px] text-slate-500 font-semibold mt-0.5 block">
                            {percentage}% of total
                          </span>
                        </div>
                      </div>

                      <div class="w-full bg-slate-950 h-1.5 rounded-full overflow-hidden">
                        <div 
                          class="h-full rounded-full bg-gradient-to-r from-violet-500 to-teal-400"
                          style={`width: ${Math.max(4, Math.min(100, (totalBytes / Math.max(1, rangeTotal)) * 100))}%`}
                        ></div>
                      </div>
                    </div>
                  {/each}

                  <!-- Pagination Controller -->
                  {#if totalNetworkPages > 1}
                    <div class="flex items-center justify-between border-t border-slate-800/60 pt-4 mt-2">
                      <button
                        class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
                        type="button"
                        disabled={networkPage === 1}
                        onclick={() => networkPage -= 1}
                      >
                        Previous
                      </button>
                      <span class="text-xs text-slate-400 font-semibold">
                        Page {networkPage} of {totalNetworkPages}
                      </span>
                      <button
                        class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
                        type="button"
                        disabled={networkPage === totalNetworkPages}
                        onclick={() => networkPage += 1}
                      >
                        Next
                      </button>
                    </div>
                  {/if}
                {:else}
                  <EmptyState
                    compact
                    icon={Search}
                    title="No traffic records"
                    message="No bandwidth consumption logged."
                  />
                {/if}
              </div>
            </section>
          </div>

        {:else if statsSubTab === "habits"}
          <!-- Productivity Habits Subtab -->
          <div class="space-y-6" transition:fade={{ duration: 100 }}>
            <!-- Habits Summary grid -->
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <MetricCard
                icon={Flame}
                tone="yellow"
                title="Active Streaks"
                value={String(timeline?.days.filter(d => d.totalSeconds > 0).length ?? 0)}
                caption="Total logged days"
              />
              <MetricCard
                icon={Target}
                tone="purple"
                title="Average Daily Time"
                value={formatDuration(averageTrackedSeconds())}
                caption="Across active days"
              />
              <MetricCard
                icon={Award}
                tone="yellow"
                title="Top Productive Day"
                value={bestTrackedDay().date ? formatDateLabel(bestTrackedDay().date) : "None"}
                caption={bestTrackedDay().totalSeconds ? formatDuration(bestTrackedDay().totalSeconds) : "No data"}
              />
              <MetricCard
                icon={Calendar}
                tone="blue"
                title="Days Tracked"
                value={String(timeline?.days.length ?? 0)}
                caption="Total database span"
              />
            </div>

            <!-- Heatmap section: Screen Time vs Network Usage -->
            <section class="silo-card p-6">
              <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-slate-800/60 pb-4 mb-6">
                <div class="flex items-center gap-3">
                  <IconBadge icon={Calendar} tone="teal" label="Activity Calendar" />
                  <div>
                    <h2 class="text-lg font-bold">Activity Heatmap</h2>
                    <p class="text-sm text-slate-500">Visualize daily usage habits for the last 12 weeks</p>
                  </div>
                </div>

                <!-- Heatmap toggle tabs -->
                <div class="flex rounded-lg bg-slate-950 p-1">
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {heatmapTab === 'screentime' ? 'bg-teal-500/20 text-teal-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => heatmapTab = "screentime"}
                  >
                    Screen Time
                  </button>
                  <button
                    class="rounded-md px-3 py-1.5 text-xs font-semibold transition
                      {heatmapTab === 'network' ? 'bg-violet-500/20 text-violet-300' : 'text-slate-400 hover:text-slate-200'}"
                    type="button"
                    onclick={() => heatmapTab = "network"}
                  >
                    Network Traffic
                  </button>
                </div>
              </div>

              {#if heatmapTab === 'screentime'}
                <!-- Screen Time Heatmap -->
                {#if timeline?.days.length}
                  <div class="mt-4 flex flex-col justify-center items-center">
                    <div class="grid grid-flow-col grid-rows-7 gap-2">
                      {#each timeline.days.slice(-84) as day}
                        <span
                          class="h-3.5 w-3.5 rounded-sm transition-all duration-200 hover:scale-125 hover:ring-2 hover:ring-teal-400/50
                            {day.totalSeconds > 14400
                              ? 'bg-teal-300 shadow-[0_0_6px_rgba(45,212,191,0.4)]'
                              : day.totalSeconds > 7200
                                ? 'bg-teal-500'
                                : day.totalSeconds > 0
                                  ? 'bg-teal-900'
                                  : 'bg-slate-900'}"
                          title={`${day.date}: ${formatDuration(day.totalSeconds)} focus time`}
                        ></span>
                      {/each}
                    </div>
                    <div class="mt-5 flex w-full max-w-sm items-center justify-between text-xs text-slate-500 px-2">
                      <span>Less (0m)</span>
                      <div class="flex gap-1">
                        <span class="h-3 w-3 bg-slate-900 rounded-sm"></span>
                        <span class="h-3 w-3 bg-teal-900 rounded-sm"></span>
                        <span class="h-3 w-3 bg-teal-500 rounded-sm"></span>
                        <span class="h-3 w-3 bg-teal-300 rounded-sm"></span>
                      </div>
                      <span>More (4h+)</span>
                    </div>
                  </div>
                {:else}
                  <EmptyState
                    icon={Calendar}
                    title="No heatmap data"
                    message="Heatmap dots will appear once focus sessions are recorded."
                  />
                {/if}
              {:else}
                <!-- Network Heatmap -->
                {#if heatmapNetworkHistory.length}
                  <div class="mt-4 flex flex-col justify-center items-center">
                    <div class="grid grid-flow-col grid-rows-7 gap-2">
                      {#each heatmapNetworkHistory.slice(-84) as day}
                        {@const dayBytes = day.downloadBytes + day.uploadBytes}
                        <span
                          class="h-3.5 w-3.5 rounded-sm transition-all duration-200 hover:scale-125 hover:ring-2 hover:ring-violet-400/50
                            {getNetworkHeatmapColor(dayBytes)}"
                          title={`${day.date}: ${formatBytes(dayBytes)} total data`}
                        ></span>
                      {/each}
                    </div>
                    <div class="mt-5 flex w-full max-w-sm items-center justify-between text-xs text-slate-500 px-2">
                      <span>Less (0B)</span>
                      <div class="flex gap-1">
                        <span class="h-3 w-3 bg-slate-900 rounded-sm"></span>
                        <span class="h-3 w-3 bg-violet-900 rounded-sm"></span>
                        <span class="h-3 w-3 bg-violet-500 rounded-sm"></span>
                        <span class="h-3 w-3 bg-violet-300 rounded-sm"></span>
                      </div>
                      <span>More (1GB+)</span>
                    </div>
                  </div>
                {:else}
                  <EmptyState
                    icon={Calendar}
                    title="No network heatmap data"
                    message="Heatmap dots will appear once network activity is recorded."
                  />
                {/if}
              {/if}
            </section>

            <!-- Focus Time weekly analysis -->
            <section class="silo-card p-6">
              <div class="flex items-center justify-between border-b border-slate-800/60 pb-4 mb-5">
                <div>
                  <h2 class="text-lg font-bold">Focus Trend</h2>
                  <p class="text-sm text-slate-500">Weekly tracked screen time (minutes)</p>
                </div>
                <span class="rounded-lg bg-slate-800 px-3 py-1.5 text-xs font-bold text-slate-300">
                  Last 7 Days
                </span>
              </div>
              {#if weekDays().length}
                <div class="mt-6">
                  <TrendChart
                    labels={chartLabels()}
                    datasets={focusChartData()}
                    height={300}
                  />
                </div>
              {:else}
                <EmptyState
                  icon={ChartColumn}
                  title="No focus trends"
                  message="Trend chart will populate once sessions are logged."
                />
              {/if}
            </section>
          </div>
        {/if}
      </section>
    {:else if activeView === "network"}
      <section class="mx-auto max-w-6xl space-y-6">
        <header
          class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
        >
          <div>
            <div class="flex items-center gap-3">
              <Wifi class="text-teal-300" size={28} />
              <h1 class="text-4xl font-black">Network</h1>
            </div>
            <p class="mt-2 text-slate-400">
              Live speed, per-app and per-site bandwidth usage
            </p>
          </div>
          <button
            class="silo-button-secondary"
            type="button"
            onclick={exportUsage}
            disabled={exporting}
          >
            <FileDown size={16} />
            Export Usage
          </button>
        </header>

        <section class="silo-card p-6">
          <div class="flex items-center justify-between text-sm text-slate-500">
            <span class="flex items-center gap-2"
              ><span class="h-2 w-2 rounded-full bg-emerald-400"></span> Live network
              speed</span
            >
            <span>Sample interval: {settings?.sampleIntervalSeconds ?? 5}s</span
            >
          </div>
          <div class="mt-8 grid gap-8 md:grid-cols-2">
            <div>
              <p class="flex items-center gap-2 text-sm text-slate-500">
                <Download size={16} /> Download
              </p>
              <p class="mt-2 text-5xl font-black text-teal-300">
                {formatBps(snapshot?.networkSpeed.downloadBps ?? 0)}
              </p>
              <p class="mt-2 text-sm text-slate-500">
                Today: {formatBytes(dataUsage?.totalDownloadBytes ?? 0)}
              </p>
            </div>
            <div>
              <p class="flex items-center gap-2 text-sm text-slate-500">
                <Upload size={16} /> Upload
              </p>
              <p class="mt-2 text-5xl font-black text-violet-400">
                {formatBps(snapshot?.networkSpeed.uploadBps ?? 0)}
              </p>
              <p class="mt-2 text-sm text-slate-500">
                Today: {formatBytes(dataUsage?.totalUploadBytes ?? 0)}
              </p>
            </div>
          </div>
          {#if liveNetworkSamples.length > 1}
            <div class="mt-8">
              <TrendChart
                labels={liveChartLabels()}
                datasets={liveChartData()}
                type="line"
                height={200}
              />
            </div>
          {:else}
            <div class="mt-8 h-1 rounded-full bg-teal-400/70"></div>
          {/if}
        </section>

        <section class="silo-card p-6">
          <div
            class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
          >
            <div>
              <h2 class="text-lg font-bold">Usage breakdown</h2>
              <p class="text-sm text-slate-500">
                See how much data each app and site used over the selected
                period
              </p>
            </div>
            <div class="flex rounded-lg bg-slate-800 p-1">
              {#each ["7d", "30d", "90d"] as range}
                <button
                  class="rounded-md px-3 py-2 text-xs font-bold transition {dataRange ===
                  range
                    ? 'bg-slate-950 text-slate-100'
                    : 'text-slate-400 hover:text-slate-100'}"
                  type="button"
                  onclick={() => loadDataUsage(range as RangeKey)}
                >
                  {range}
                </button>
              {/each}
            </div>
          </div>
          <div class="mt-6 grid gap-5 sm:grid-cols-4">
            <div>
              <p class="text-sm text-slate-500">Total</p>
              <p class="mt-1 text-2xl font-black">
                {formatBytes(totalDataBytes())}
              </p>
            </div>
            <div>
              <p class="text-sm text-slate-500">Download</p>
              <p class="mt-1 text-2xl font-black">
                {formatBytes(dataUsage?.totalDownloadBytes ?? 0)}
              </p>
            </div>
            <div>
              <p class="text-sm text-slate-500">Upload</p>
              <p class="mt-1 text-2xl font-black">
                {formatBytes(dataUsage?.totalUploadBytes ?? 0)}
              </p>
            </div>
            <div>
              <p class="text-sm text-slate-500">Range</p>
              <p class="mt-1 text-2xl font-black">{dataRange}</p>
            </div>
          </div>
          {#if totalDataBytes() > 0}
            <div class="mt-6">
              <TrendChart
                labels={["Selected range"]}
                datasets={networkChartData()}
                height={220}
              />
            </div>
          {/if}
        </section>

        <div class="grid gap-5 lg:grid-cols-2">
          {@render ConsumerList(
            "Top Apps",
            dataUsage?.apps ?? [],
            "Apps",
            Monitor,
          )}
          {@render ConsumerList(
            "Top Sites",
            dataUsage?.sites ?? [],
            "Websites",
            Globe,
          )}
        </div>

        <section class="silo-card p-6">
          <div
            class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
          >
            <div>
              <h2 class="text-lg font-bold">Usage History</h2>
              <p class="text-sm text-slate-500">
                Total data usage (upload + download) per day
              </p>
            </div>
            <div class="flex rounded-lg bg-slate-800 p-1">
              {#each ["7d", "30d"] as range}
                <button
                  class="rounded-md px-3 py-2 text-xs font-bold transition {networkHistoryRange ===
                  range
                    ? 'bg-slate-950 text-slate-100'
                    : 'text-slate-400 hover:text-slate-100'}"
                  type="button"
                  onclick={() =>
                    changeNetworkHistoryRange(range as "7d" | "30d")}
                >
                  {range === "7d" ? "Weekly" : "Monthly"}
                </button>
              {/each}
            </div>
          </div>
          {#if networkHistory.length}
            <div class="mt-6">
              <TrendChart
                labels={historyChartLabels()}
                datasets={historyChartData()}
                type="bar"
                height={260}
              />
            </div>
          {:else}
            <EmptyState
              icon={HardDrive}
              title="No history logged yet"
              message="Attributed daily network volume will appear here as SILO measures background traffic."
            />
          {/if}
        </section>
      </section>
    {:else if activeView === "network-apps"}
      <section
        class="mx-auto max-w-5xl space-y-6"
        transition:fade={{ duration: 150 }}
      >
        <!-- Header Nav Row -->
        <header
          class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between"
        >
          <div class="flex items-center gap-4">
            <button
              class="inline-flex items-center gap-2 text-slate-300 hover:text-slate-100 bg-slate-800 hover:bg-slate-700 px-4 py-2.5 rounded-xl font-semibold transition hover:scale-[1.02] active:scale-[0.98] border border-slate-700/50"
              type="button"
              onclick={() => (activeView = "network")}
            >
              ← Back
            </button>
            <div>
              <h1 class="text-3xl font-black flex items-center gap-2">
                <Monitor class="text-teal-350 shrink-0" size={24} />
                All App Usages
              </h1>
              <p class="text-sm text-slate-505 mt-0.5">
                Detailed metrics of local applications
              </p>
            </div>
          </div>

          <!-- Date Range Pills -->
          <div
            class="flex rounded-xl bg-slate-900/90 border border-slate-800 p-1 w-fit shadow-lg shadow-black/20 self-start md:self-auto"
          >
            {#each [{ key: "today", label: "Today" }, { key: "7d", label: "Last 7 Days" }, { key: "30d", label: "Last 30 Days" }] as range}
              <button
                class="rounded-lg px-4 py-2 text-xs font-bold transition-all duration-200 {detailRange ===
                range.key
                  ? 'bg-teal-400 text-slate-950 font-black shadow-md'
                  : 'text-slate-400 hover:text-slate-100'}"
                type="button"
                onclick={() =>
                  loadDetailUsage(range.key as "today" | "7d" | "30d")}
              >
                {range.label}
              </button>
            {/each}
          </div>
        </header>

        <!-- Stats Overview Cards Grid -->
        <div class="grid gap-4 grid-cols-2 lg:grid-cols-4">
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-500 uppercase tracking-wider"
              >Total Usage</span
            >
            <span class="text-2xl font-black text-slate-200 mt-2 block"
              >{formatBytes(getDetailTotalBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Upload + Download</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1"
            >
              <Download class="text-teal-400" size={13} />
              Download
            </span>
            <span class="text-2xl font-black text-teal-350 mt-2 block"
              >{formatBytes(getDetailDownloadBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Received data</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1"
            >
              <Upload class="text-violet-400" size={13} />
              Upload
            </span>
            <span class="text-2xl font-black text-violet-300 mt-2 block"
              >{formatBytes(getDetailUploadBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Sent data</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider"
              >Active Apps</span
            >
            <span class="text-2xl font-black text-slate-200 mt-2 block"
              >{filteredMoreRows.length}</span
            >
            <span class="text-xs text-slate-505 mt-1"
              >Attributed this period</span
            >
          </div>
        </div>

        <!-- Main Content List Card -->
        <section class="silo-card p-6 space-y-6">
          <!-- Search Bar -->
          <div class="relative w-full">
            <Search
              class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-550"
              size={17}
            />
            <input
              class="silo-input pl-10.5 w-full bg-slate-950/40 focus:border-teal-400 transition-colors"
              placeholder="Search apps..."
              bind:value={moreModalSearch}
            />
          </div>

          <!-- Table Headers (Desktop) -->
          <div
            class="hidden sm:grid grid-cols-[1.5fr_1fr_1fr_1fr] px-4 py-2 border-b border-slate-800/60 text-xs font-extrabold text-slate-550 tracking-wider"
          >
            <span>APPLICATION</span>
            <span class="text-right flex items-center justify-end gap-1"
              ><Download size={11} /> DOWNLOAD</span
            >
            <span class="text-right flex items-center justify-end gap-1"
              ><Upload size={11} /> UPLOAD</span
            >
            <span class="text-right">TOTAL</span>
          </div>

          <div class="space-y-3.5">
            {#if filteredMoreRows.length}
              {#each filteredMoreRows as row, i}
                {@const totalBytes = consumerTotal(row)}
                {@const sharePercentage = (
                  (totalBytes / Math.max(1, getDetailTotalBytes())) *
                  105
                ).toFixed(1)}
                <div
                  class="silo-card p-4 hover:border-slate-700/85 hover:bg-slate-900/30 transition-all duration-200 group flex flex-col gap-3"
                >
                  <!-- Row top info -->
                  <div
                    class="flex flex-col sm:grid sm:grid-cols-[1.5fr_1fr_1fr_1fr] items-start sm:items-center justify-between gap-2.5"
                  >
                    <div class="flex items-center gap-3 min-w-0 w-full">
                      <!-- Rank badge -->
                      <span
                        class="text-xs font-black px-2 py-1 bg-slate-800 text-slate-400 group-hover:bg-teal-400 group-hover:text-slate-950 rounded-md transition-colors w-7 text-center"
                      >
                        #{i + 1}
                      </span>
                      <p
                        class="truncate font-black text-slate-200 group-hover:text-teal-300 transition-colors"
                      >
                        {row.name}
                      </p>
                    </div>

                    <!-- Down bytes -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right"
                    >
                      <span class="sm:hidden font-semibold text-slate-500"
                        >Download</span
                      >
                      <span class="font-semibold text-teal-350/90"
                        >{formatBytes(row.downloadBytes)}</span
                      >
                    </div>

                    <!-- Up bytes -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right"
                    >
                      <span class="sm:hidden font-semibold text-slate-500"
                        >Upload</span
                      >
                      <span class="font-semibold text-violet-350"
                        >{formatBytes(row.uploadBytes)}</span
                      >
                    </div>

                    <!-- Total bytes & Share % -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end items-center gap-2 sm:text-right font-black text-slate-100"
                    >
                      <span class="sm:hidden font-bold text-slate-500"
                        >Total</span
                      >
                      <div class="flex items-baseline gap-1.5 justify-end">
                        <span class="text-slate-100"
                          >{formatBytes(totalBytes)}</span
                        >
                        <span class="text-xs text-slate-500 font-semibold"
                          >({sharePercentage}%)</span
                        >
                      </div>
                    </div>
                  </div>

                  <!-- Distribution Progress Bar -->
                  <div class="h-2 rounded-full bg-slate-950/70 overflow-hidden">
                    <div
                      class="h-full rounded-full bg-gradient-to-r from-teal-400 to-violet-500 transition-all duration-500"
                      style={`width: ${Math.max(4, Math.min(100, (totalBytes / Math.max(1, getDetailTotalBytes())) * 105))}%`}
                    ></div>
                  </div>
                </div>
              {/each}
            {:else}
              <EmptyState
                compact
                icon={Search}
                title="No results found"
                message="Try a different search query."
              />
            {/if}
          </div>
        </section>
      </section>
    {:else if activeView === "network-sites"}
      <section
        class="mx-auto max-w-5xl space-y-6"
        transition:fade={{ duration: 150 }}
      >
        <!-- Header Nav Row -->
        <header
          class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between"
        >
          <div class="flex items-center gap-4">
            <button
              class="inline-flex items-center gap-2 text-slate-300 hover:text-slate-100 bg-slate-800 hover:bg-slate-700 px-4 py-2.5 rounded-xl font-semibold transition hover:scale-[1.02] active:scale-[0.98] border border-slate-700/50"
              type="button"
              onclick={() => (activeView = "network")}
            >
              ← Back
            </button>
            <div>
              <h1 class="text-3xl font-black flex items-center gap-2">
                <Globe class="text-teal-350 shrink-0" size={24} />
                All Site Usages
              </h1>
              <p class="text-sm text-slate-505 mt-0.5">
                Detailed metrics of visited websites
              </p>
            </div>
          </div>

          <!-- Date Range Pills -->
          <div
            class="flex rounded-xl bg-slate-900/90 border border-slate-800 p-1 w-fit shadow-lg shadow-black/20 self-start md:self-auto"
          >
            {#each [{ key: "today", label: "Today" }, { key: "7d", label: "Last 7 Days" }, { key: "30d", label: "Last 30 Days" }] as range}
              <button
                class="rounded-lg px-4 py-2 text-xs font-bold transition-all duration-200 {detailRange ===
                range.key
                  ? 'bg-teal-400 text-slate-950 font-black shadow-md'
                  : 'text-slate-400 hover:text-slate-100'}"
                type="button"
                onclick={() =>
                  loadDetailUsage(range.key as "today" | "7d" | "30d")}
              >
                {range.label}
              </button>
            {/each}
          </div>
        </header>

        <!-- Stats Overview Cards Grid -->
        <div class="grid gap-4 grid-cols-2 lg:grid-cols-4">
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-500 uppercase tracking-wider"
              >Total Usage</span
            >
            <span class="text-2xl font-black text-slate-200 mt-2 block"
              >{formatBytes(getDetailTotalBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Upload + Download</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1"
            >
              <Download class="text-teal-400" size={13} />
              Download
            </span>
            <span class="text-2xl font-black text-teal-350 mt-2 block"
              >{formatBytes(getDetailDownloadBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Received data</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1"
            >
              <Upload class="text-violet-400" size={13} />
              Upload
            </span>
            <span class="text-2xl font-black text-violet-300 mt-2 block"
              >{formatBytes(getDetailUploadBytes())}</span
            >
            <span class="text-xs text-slate-505 mt-1">Sent data</span>
          </div>
          <div
            class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80"
          >
            <span
              class="text-xs font-bold text-slate-550 uppercase tracking-wider"
              >Active Sites</span
            >
            <span class="text-2xl font-black text-slate-200 mt-2 block"
              >{filteredMoreRows.length}</span
            >
            <span class="text-xs text-slate-505 mt-1"
              >Attributed this period</span
            >
          </div>
        </div>

        <!-- Main Content List Card -->
        <section class="silo-card p-6 space-y-6">
          <!-- Search Bar -->
          <div class="relative w-full">
            <Search
              class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-550"
              size={17}
            />
            <input
              class="silo-input pl-10.5 w-full bg-slate-950/40 focus:border-teal-400 transition-colors"
              placeholder="Search sites..."
              bind:value={moreModalSearch}
            />
          </div>

          <!-- Table Headers (Desktop) -->
          <div
            class="hidden sm:grid grid-cols-[1.5fr_1fr_1fr_1fr] px-4 py-2 border-b border-slate-800/60 text-xs font-extrabold text-slate-550 tracking-wider"
          >
            <span>WEBSITE</span>
            <span class="text-right flex items-center justify-end gap-1"
              ><Download size={11} /> DOWNLOAD</span
            >
            <span class="text-right flex items-center justify-end gap-1"
              ><Upload size={11} /> UPLOAD</span
            >
            <span class="text-right">TOTAL</span>
          </div>

          <div class="space-y-3.5">
            {#if filteredMoreRows.length}
              {#each filteredMoreRows as row, i}
                {@const totalBytes = consumerTotal(row)}
                {@const sharePercentage = (
                  (totalBytes / Math.max(1, getDetailTotalBytes())) *
                  105
                ).toFixed(1)}
                <div
                  class="silo-card p-4 hover:border-slate-700/85 hover:bg-slate-900/30 transition-all duration-200 group flex flex-col gap-3"
                >
                  <!-- Row top info -->
                  <div
                    class="flex flex-col sm:grid sm:grid-cols-[1.5fr_1fr_1fr_1fr] items-start sm:items-center justify-between gap-2.5"
                  >
                    <div class="flex items-center gap-3 min-w-0 w-full">
                      <!-- Rank badge -->
                      <span
                        class="text-xs font-black px-2 py-1 bg-slate-800 text-slate-400 group-hover:bg-teal-400 group-hover:text-slate-950 rounded-md transition-colors w-7 text-center"
                      >
                        #{i + 1}
                      </span>
                      <p
                        class="truncate font-black text-slate-200 group-hover:text-teal-300 transition-colors"
                      >
                        {row.name}
                      </p>
                    </div>

                    <!-- Down bytes -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right"
                    >
                      <span class="sm:hidden font-semibold text-slate-500"
                        >Download</span
                      >
                      <span class="font-semibold text-teal-350/90"
                        >{formatBytes(row.downloadBytes)}</span
                      >
                    </div>

                    <!-- Up bytes -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right"
                    >
                      <span class="sm:hidden font-semibold text-slate-500"
                        >Upload</span
                      >
                      <span class="font-semibold text-violet-350"
                        >{formatBytes(row.uploadBytes)}</span
                      >
                    </div>

                    <!-- Total bytes & Share % -->
                    <div
                      class="flex justify-between w-full sm:w-auto sm:justify-end items-center gap-2 sm:text-right font-black text-slate-100"
                    >
                      <span class="sm:hidden font-bold text-slate-500"
                        >Total</span
                      >
                      <div class="flex items-baseline gap-1.5 justify-end">
                        <span class="text-slate-100"
                          >{formatBytes(totalBytes)}</span
                        >
                        <span class="text-xs text-slate-500 font-semibold"
                          >({sharePercentage}%)</span
                        >
                      </div>
                    </div>
                  </div>

                  <!-- Distribution Progress Bar -->
                  <div class="h-2 rounded-full bg-slate-950/70 overflow-hidden">
                    <div
                      class="h-full rounded-full bg-gradient-to-r from-teal-400 to-violet-500 transition-all duration-500"
                      style={`width: ${Math.max(4, Math.min(100, (totalBytes / Math.max(1, getDetailTotalBytes())) * 105))}%`}
                    ></div>
                  </div>
                </div>
              {/each}
            {:else}
              <EmptyState
                compact
                icon={Search}
                title="No results found"
                message="Try a different search query."
              />
            {/if}
          </div>
        </section>
      </section>
    {:else if activeView === "settings" && settings}
      <section class="mx-auto max-w-3xl space-y-6">
        <header>
          <h1 class="text-4xl font-black">Settings</h1>
          <p class="mt-2 text-slate-400">Customize your SILO experience</p>
        </header>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Power} tone="teal" label="General" />
            <h2 class="text-lg font-bold">General</h2>
          </div>
          <div class="mt-6 space-y-5">
            {@render SettingToggle(
              "Launch at startup",
              "Start SILO when you log in",
              settings.autoStart,
              (checked) => (settings = { ...settings!, autoStart: checked }),
            )}
            {@render SettingToggle(
              "Notifications",
              "Get time limit warnings and summaries",
              settings.notificationsEnabled,
              (checked) =>
                (settings = { ...settings!, notificationsEnabled: checked }),
            )}
            {@render SettingToggle(
              "Keyboard Shortcuts",
              "Use keyboard keys for navigation and controls",
              settings.shortcutsEnabled,
              (checked) =>
                (settings = { ...settings!, shortcutsEnabled: checked }),
            )}
          </div>
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Calendar} tone="teal" label="Retention" />
            <h2 class="text-lg font-bold">Data Retention</h2>
          </div>
          <div class="mt-6 grid gap-4 sm:grid-cols-2">
            <label class="text-sm font-semibold text-slate-300">
              Detailed history retention
              <input
                class="silo-input mt-2"
                min="1"
                type="number"
                bind:value={settings.retentionDays}
              />
            </label>
            <label class="text-sm font-semibold text-slate-300">
              Network polling interval
              <input
                class="silo-input mt-2"
                min="1"
                type="number"
                bind:value={settings.sampleIntervalSeconds}
              />
            </label>
          </div>
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Moon} tone="purple" label="Appearance" />
            <h2 class="text-lg font-bold">Appearance</h2>
          </div>
          <div class="mt-6">
            <label class="text-sm font-semibold text-slate-300">
              Color theme
              <select class="silo-input mt-2" bind:value={settings.theme}>
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </label>
            <p class="mt-3 text-sm text-slate-500">
              The UI uses the dark SILO shell in this implementation pass.
            </p>
          </div>
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Database} tone="teal" label="Privacy and data" />
            <h2 class="text-lg font-bold">Privacy &amp; Data</h2>
          </div>
          <div class="mt-6 space-y-3">
            <button
              class="silo-button-secondary w-full justify-center"
              type="button"
              onclick={exportUsage}
              disabled={exporting}
            >
              <Download size={16} />
              Export Usage Data
            </button>
            <button
              class="silo-button-secondary w-full justify-center"
              type="button"
              onclick={exportLogs}
              disabled={exporting}
            >
              <FileDown size={16} />
              Export Logs
            </button>
            <button
              class="silo-button-secondary w-full justify-center"
              type="button"
              onclick={completeBackup}
            >
              <RotateCcw size={16} />
              Mark Backup Complete
            </button>
          </div>
          {#if settings.lastBackupAt}
            <p class="mt-4 text-sm text-slate-500">
              Last backup: {settings.lastBackupAt}
            </p>
          {/if}
          {#if exportPath}
            <p
              class="mt-4 break-all rounded-lg bg-slate-950/60 p-3 text-sm text-slate-400"
            >
              {exportPath}
            </p>
          {/if}
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center justify-between gap-3 border-b border-slate-800/60 pb-4 mb-5">
            <div class="flex items-center gap-3">
              <IconBadge
                icon={Keyboard}
                tone="neutral"
                label="Keyboard shortcuts"
              />
              <div>
                <h2 class="text-lg font-bold">Keyboard Shortcuts</h2>
                <p class="text-xs text-slate-500">Quickly navigate and control SILO</p>
              </div>
            </div>
            <span class="text-xs font-bold uppercase tracking-wider px-2.5 py-1 rounded-full border 
              {settings.shortcutsEnabled ? 'bg-teal-500/10 text-teal-300 border-teal-500/20' : 'bg-slate-800 text-slate-400 border-transparent'}">
              {settings.shortcutsEnabled ? 'Enabled' : 'Disabled'}
            </span>
          </div>

          <div class="grid gap-3 sm:grid-cols-2">
            {#each [
              { keys: ["D", "1"], desc: "Go to Dashboard" },
              { keys: ["R", "2"], desc: "Go to Rules & Limits" },
              { keys: ["S", "3"], desc: "Go to Statistics" },
              { keys: ["N", "4"], desc: "Go to Network Usage" },
              { keys: ["P", "5"], desc: "Go to Settings" },
              { keys: ["Space"], desc: "Toggle Focus Mode" },
              { keys: ["Esc"], desc: "Close drawers / Blur / Overlays" }
            ] as shortcut}
              <div class="flex items-center justify-between p-3 rounded-lg bg-slate-950/45 border border-slate-900">
                <span class="text-sm text-slate-300 font-semibold">{shortcut.desc}</span>
                <div class="flex items-center gap-1 shrink-0">
                  {#each shortcut.keys as key, i}
                    {#if i > 0}
                      <span class="text-[10px] text-slate-600 font-extrabold">/</span>
                    {/if}
                    <kbd class="px-2 py-1 text-xs font-black font-mono bg-slate-800 border border-slate-700/80 rounded shadow-md text-slate-100 min-w-[24px] text-center block">
                      {key}
                    </kbd>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <section class="flex justify-end">
          <button
            class="silo-button"
            type="button"
            onclick={saveSettings}
            disabled={savingSettings}
          >
            {savingSettings ? "Saving" : "Save Settings"}
          </button>
        </section>
      </section>
    {/if}
  </div>

  <BottomNav
    items={navItems}
    active={activeView === "network-apps" || activeView === "network-sites"
      ? "network"
      : activeView}
    onSelect={(key) => (activeView = key as ViewKey)}
  />

  {#if showWarningOverlay && warningData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md"
    >
      <div
        transition:fly={{ y: 25, duration: 250 }}
        class="silo-card max-w-md w-full p-8 border border-amber-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6"
      >
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-amber-500/10 text-amber-400"
        >
          <CircleAlert size={32} />
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-black text-slate-100">
            Limit Warning
          </h2>
          <p class="text-sm text-slate-400">
            You have <span class="text-amber-300 font-bold">{formatDuration(warningData.remainingSeconds)}</span> remaining today for this {warningData.ruleType}.
          </p>
        </div>
        <div
          class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono"
        >
          <span class="text-amber-300 font-bold">{warningData.target}</span>
          <div class="text-xs text-slate-500 mt-1">
            Total Limit: {formatDuration(warningData.limitSeconds)}
          </div>
        </div>
        <div class="space-y-3">
          <p class="text-xs text-slate-500 font-semibold uppercase tracking-wider">Add time for today</p>
          <div class="flex flex-wrap justify-center gap-2">
            <button
              class="silo-button-secondary bg-slate-800 hover:bg-slate-700 text-slate-200 px-4 py-2 text-xs rounded-lg transition"
              type="button"
              onclick={() => warningData && addTime(warningData.ruleId, 15)}
            >
              +15 Min
            </button>
            <button
              class="silo-button-secondary bg-slate-800 hover:bg-slate-700 text-slate-200 px-4 py-2 text-xs rounded-lg transition"
              type="button"
              onclick={() => warningData && addTime(warningData.ruleId, 30)}
            >
              +30 Min
            </button>
            <button
              class="silo-button-secondary bg-slate-800 hover:bg-slate-700 text-slate-200 px-4 py-2 text-xs rounded-lg transition"
              type="button"
              onclick={() => warningData && addTime(warningData.ruleId, 60)}
            >
              +1 Hour
            </button>
          </div>
        </div>
        <div class="flex justify-center pt-2">
          <button
            class="silo-button-secondary hover:bg-slate-800 text-slate-400 font-bold px-6 py-2 rounded-lg transition"
            type="button"
            onclick={() => {
              showWarningOverlay = false;
              warningData = null;
            }}
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showCountdownOverlay && countdownData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/90 backdrop-blur-lg"
    >
      <div
        transition:fly={{ y: 25, duration: 250 }}
        class="silo-card max-w-md w-full p-8 border border-red-500/40 bg-slate-900/95 shadow-2xl text-center space-y-6"
      >
        <div
          class="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-red-500/10 text-red-500 animate-pulse"
        >
          <span class="text-3xl font-black font-mono">{countdownData.remainingSeconds}</span>
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-black text-red-400">
            {countdownData.enforcement === 'hard' ? 'Hard Block' : 'Limit'} Closing App
          </h2>
          <p class="text-sm text-slate-400">
            Save your work immediately! The application will close.
          </p>
        </div>
        <div
          class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono"
        >
          <span class="text-red-300 font-bold">{countdownData.target}</span>
        </div>

        {#if countdownData.enforcement === 'warn'}
          <div class="space-y-3">
            <p class="text-xs text-slate-500 font-semibold uppercase tracking-wider">Add time for today</p>
            <div class="flex flex-wrap justify-center gap-2">
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => countdownData && addTime(countdownData.ruleId, 15)}
              >
                +15 Min
              </button>
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => countdownData && addTime(countdownData.ruleId, 30)}
              >
                +30 Min
              </button>
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => countdownData && addTime(countdownData.ruleId, 60)}
              >
                +1 Hour
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if showViolationOverlay && violationData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md"
    >
      <div
        transition:fly={{ y: 25, duration: 250 }}
        class="silo-card max-w-md w-full p-8 border border-red-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6"
      >
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10 text-red-400"
        >
          <CircleAlert size={32} />
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-black text-slate-100">
            {violationData.enforcement === 'soft' ? 'Focus Limit Reached' : 'Focus Limit Reached & Closed'}
          </h2>
          <p class="text-sm text-slate-400">
            You've reached the daily limit for this {violationData.ruleType}.
          </p>
        </div>
        <div
          class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono"
        >
          <span class="text-red-300 font-bold">{violationData.target}</span>
          <div class="text-xs text-slate-500 mt-1">
            Daily Limit: {formatDuration(violationData.limitSeconds)}
          </div>
        </div>

        {#if violationData.enforcement === 'warn'}
          <div class="space-y-3">
            <p class="text-xs text-slate-500 font-semibold uppercase tracking-wider">Add time for today</p>
            <div class="flex flex-wrap justify-center gap-2">
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => violationData && addTime(violationData.ruleId, 15)}
              >
                +15 Min
              </button>
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => violationData && addTime(violationData.ruleId, 30)}
              >
                +30 Min
              </button>
              <button
                class="silo-button bg-teal-600 hover:bg-teal-700 text-white px-4 py-2 text-xs rounded-lg transition"
                type="button"
                onclick={() => violationData && addTime(violationData.ruleId, 60)}
              >
                +1 Hour
              </button>
            </div>
          </div>
        {/if}

        <div class="flex justify-center gap-3 pt-2">
          <button
            class="silo-button bg-red-600 hover:bg-red-700 text-white font-bold px-6 py-2.5 rounded-lg transition"
            type="button"
            onclick={() => {
              showViolationOverlay = false;
              violationData = null;
            }}
          >
            Acknowledge
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showConfirmDeleteOverlay && ruleToDelete}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md"
    >
      <div
        transition:fly={{ y: 25, duration: 250 }}
        class="silo-card max-w-md w-full p-8 border border-red-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6"
      >
        <div
          class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10 text-red-400"
        >
          <Trash2 size={32} />
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-black text-slate-100">
            Delete Rule
          </h2>
          <p class="text-sm text-slate-400">
            Are you sure you want to delete the rule for this {ruleToDelete.ruleType}?
          </p>
        </div>
        <div
          class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono"
        >
          <span class="text-red-300 font-bold">{ruleToDelete.target}</span>
        </div>

        <div class="flex justify-center gap-3 pt-2">
          <button
            class="silo-button-secondary bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold px-6 py-2.5 rounded-lg transition"
            type="button"
            onclick={cancelDeleteRule}
          >
            Cancel
          </button>
          <button
            class="silo-button bg-red-600 hover:bg-red-700 text-white font-bold px-6 py-2.5 rounded-lg transition"
            type="button"
            onclick={confirmDeleteRule}
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Toast Notification Container -->
  {#if toasts.length}
    <div
      class="fixed top-6 right-6 z-[1000] flex flex-col gap-3 max-w-sm w-full pointer-events-none"
    >
      {#each toasts as toast (toast.id)}
        <div
          transition:fly={{ x: 100, duration: 300 }}
          class="pointer-events-auto p-4 rounded-xl border backdrop-blur-md shadow-lg flex items-center gap-3 text-sm font-semibold transition-all
            {toast.type === 'success'
            ? 'border-emerald-500/30 bg-emerald-950/80 text-emerald-300'
            : toast.type === 'error'
              ? 'border-red-500/30 bg-red-950/80 text-red-300'
              : 'border-blue-500/30 bg-blue-950/80 text-blue-300'}"
        >
          <div class="flex-1">{toast.message}</div>
        </div>
      {/each}
    </div>
  {/if}
</main>

{#snippet ConsumerList(
  title: string,
  rows: DataConsumer[],
  label: string,
  icon: any,
)}
  <section class="silo-card p-6 flex flex-col justify-between">
    <div>
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <IconBadge {icon} tone="teal" label={title} />
          <h2 class="text-lg font-bold">{title}</h2>
        </div>
        <span class="text-xs text-slate-500">{dataRange}</span>
      </div>
      <div class="mt-6 space-y-5">
        {#if rows.length}
          {#each rows.slice(0, 5) as row}
            <div>
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="truncate font-bold">{row.name}</p>
                  <p class="text-sm text-slate-500">
                    Down {formatBytes(row.downloadBytes)} · Up {formatBytes(
                      row.uploadBytes,
                    )}
                  </p>
                </div>
                <p class="shrink-0 text-sm font-bold">
                  {formatBytes(consumerTotal(row))}
                </p>
              </div>
              <div class="mt-2 h-2 rounded-full bg-slate-800">
                <div
                  class="h-2 rounded-full bg-teal-400"
                  style={`width: ${Math.max(4, Math.min(100, (consumerTotal(row) / Math.max(1, totalDataBytes())) * 100))}%`}
                ></div>
              </div>
            </div>
          {/each}
        {:else}
          <EmptyState
            compact
            icon={HardDrive}
            title="No usage records"
            message="Per-app and per-site data usage will appear after attribution is implemented."
          />
        {/if}
      </div>
    </div>
    {#if rows.length > 5}
      <div class="mt-6 border-t border-slate-800 pt-4 text-center">
        <button
          class="silo-button-secondary w-full justify-center py-2"
          type="button"
          onclick={() => openMoreScreen(title, rows)}
        >
          View all {label}
        </button>
      </div>
    {/if}
  </section>
{/snippet}

{#snippet SettingToggle(
  title: string,
  description: string,
  checked: boolean,
  onChange: (checked: boolean) => void,
)}
  <div class="flex items-center justify-between gap-4">
    <div>
      <p class="font-semibold">{title}</p>
      <p class="mt-1 text-sm text-slate-500">{description}</p>
    </div>
    <ToggleSwitch {checked} label={title} onchange={onChange} />
  </div>
{/snippet}
