<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import {
    Database,
    CircleAlert,
    Timer,
    House,
    Shield,
    ChartColumn,
    Wifi,
    Settings as SettingsIcon,
  } from "lucide-svelte";
  import BottomNav from "$lib/components/BottomNav.svelte";
  import {
    emptyRule,
    siloApi,
    type AppSnapshot,
    type BootStatus,
    type Rule,
    type Settings,
    type UsageReport,
    type UsageTimeline,
    type DataUsageReport,
    type UsageDayBytes,
    type DataConsumer,
  } from "$lib/siloApi";

  // Import views
  import DashboardView from "$lib/components/views/DashboardView.svelte";
  import RulesView from "$lib/components/views/RulesView.svelte";
  import StatsView from "$lib/components/views/StatsView.svelte";
  import NetworkView from "$lib/components/views/NetworkView.svelte";
  import NetworkAppsView from "$lib/components/views/NetworkAppsView.svelte";
  import NetworkSitesView from "$lib/components/views/NetworkSitesView.svelte";
  import SettingsView from "$lib/components/views/SettingsView.svelte";

  type ViewKey =
    | "dashboard"
    | "rules"
    | "stats"
    | "network"
    | "settings"
    | "network-apps"
    | "network-sites";
  type RangeKey = "today" | "7d" | "30d" | "90d";

  const navItems = [
    { key: "dashboard", label: "Dashboard", icon: "House" }, // Note: BottomNav expects string keys if matching, but bottom nav handles it.
  ];

  // Core reactive states
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
  let dataRange = $state<RangeKey>("30d");

  let loading = $state(true);
  let exporting = $state(false);
  let exportPath = $state("");

  let rulesAuthorized = $state(false);

  // Overlays state
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

  // Network detailed states
  let detailRange = $state<"today" | "7d" | "30d">("today");
  let detailUsage = $state<DataUsageReport | null>(null);
  let moreModalSearch = $state("");

  let liveNetworkSamples = $state<
    Array<{ time: string; down: number; up: number }>
  >([]);
  let networkHistoryRange = $state<"7d" | "30d">("7d");
  let networkHistory = $state<UsageDayBytes[]>([]);

  // Toast structures
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

  // Swipe gesture variables
  let touchStartX = 0;
  let touchStartY = 0;
  let lastSwipeTime = 0;

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
      const currentIndex = views.indexOf(activeView as any);
      if (diffX > 0) {
        if (currentIndex > 0) activeView = views[currentIndex - 1];
      } else {
        if (currentIndex < views.length - 1) activeView = views[currentIndex + 1];
      }
    }
  }

  function handleWheel(e: WheelEvent) {
    if (Math.abs(e.deltaX) > 35) {
      const now = Date.now();
      if (now - lastSwipeTime < 800) return;

      const views: ViewKey[] = [
        "dashboard",
        "rules",
        "stats",
        "network",
        "settings",
      ];
      const currentIndex = views.indexOf(activeView as any);

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

  // Tauri listeners setup
  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    void loadAll();
    const timer = window.setInterval(() => void refreshLiveState(), 5000);

    getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        rulesAuthorized = false;
      }
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

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

  // Loader calls
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
        loadNetworkHistory(),
      ]);
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      loading = false;
    }
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
  }

  async function loadDataUsage(range: RangeKey) {
    dataRange = range;
    dataUsage = await siloApi.getDataUsage(range);
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

  // Toggles and Actions
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

  // General keyboard navigation
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
      }
      return;
    }

    if (e.key === "Escape") {
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
      if (e.altKey) {
        switch (e.key.toLowerCase()) {
          case "d":
            activeView = "dashboard";
            e.preventDefault();
            break;
          case "r":
            activeView = "rules";
            e.preventDefault();
            break;
          case "s":
            activeView = "stats";
            e.preventDefault();
            break;
          case "n":
            activeView = "network";
            e.preventDefault();
            break;
          case "p":
          case ",":
            activeView = "settings";
            e.preventDefault();
            break;
        }
      } else if (e.key === " ") {
        void toggleFocus();
        e.preventDefault();
      }
    }
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function formatDuration(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m`;
    return `${safeSeconds}s`;
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<svelte:head>
  <title>SILO</title>
</svelte:head>

<main
  class="min-h-screen bg-silo text-slate-100 font-sans"
  ontouchstart={handleTouchStart}
  ontouchend={handleTouchEnd}
  onwheel={handleWheel}
>
  <div class="mx-auto min-h-screen w-full max-w-7xl px-4 pb-28 pt-8 sm:px-6 lg:px-8">
    {#if loading}
      <section class="flex min-h-[70vh] items-center justify-center">
        <div class="text-center">
          <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-teal-400/15 text-teal-300">
            <Database size={26} />
          </div>
          <p class="mt-4 text-lg font-bold">Loading SILO workspace</p>
          <p class="mt-1 text-sm text-slate-500">
            Connecting to local monitoring services.
          </p>
        </div>
      </section>
    {:else}
      <!-- Render views based on activeView -->
      {#if activeView === "dashboard"}
        <DashboardView
          {snapshot}
          {boot}
          {rules}
          bind:usage
          bind:usageTab
          bind:isRefreshing
          bind:refreshKey
          {timeline}
          {dataUsage}
          {networkHistory}
          {liveNetworkSamples}
          {toggleFocus}
          {handleRefresh}
          {openMoreScreen}
        />
      {:else if activeView === "rules"}
        <RulesView
          bind:rules
          bind:snapshot
          {settings}
          {usage}
          bind:activeView
          {showToast}
          bind:isAuthorized={rulesAuthorized}
        />
      {:else if activeView === "stats"}
        <StatsView
          {rules}
          ruleDraft={emptyRule()}
          showRuleForm={false}
          bind:activeView
        />
      {:else if activeView === "network"}
        <NetworkView
          {snapshot}
          {settings}
          {dataUsage}
          exportUsage={async () => {
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
          }}
          {exporting}
          bind:dataRange
          loadDataUsage={async (range) => {
            dataRange = range;
            dataUsage = await siloApi.getDataUsage(range);
          }}
          {liveNetworkSamples}
          bind:networkHistoryRange
          changeNetworkHistoryRange={changeNetworkHistoryRange}
          {networkHistory}
          {openMoreScreen}
        />
      {:else if activeView === "network-apps"}
        <NetworkAppsView
          bind:activeView
          bind:detailRange
          loadDetailUsage={loadDetailUsage}
          {detailUsage}
          bind:moreModalSearch
        />
      {:else if activeView === "network-sites"}
        <NetworkSitesView
          bind:activeView
          bind:detailRange
          loadDetailUsage={loadDetailUsage}
          {detailUsage}
          bind:moreModalSearch
        />
      {:else if activeView === "settings"}
        <SettingsView
          bind:settings
          bind:boot
          {showToast}
        />
      {/if}
    {/if}
  </div>

  <!-- Bottom Navigation Bar -->
  <BottomNav
    items={[
      { key: "dashboard", label: "Dashboard", icon: House },
      { key: "rules", label: "Rules", icon: Shield },
      { key: "stats", label: "Stats", icon: ChartColumn },
      { key: "network", label: "Network", icon: Wifi },
      { key: "settings", label: "Settings", icon: SettingsIcon },
    ]}
    active={activeView === "network-apps" || activeView === "network-sites"
      ? "network"
      : activeView}
    onSelect={(key) => (activeView = key as ViewKey)}
  />

  <!-- Global warning dialog -->
  {#if showWarningOverlay && warningData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md"
    >
      <div
        class="silo-card max-w-md w-full p-8 border border-amber-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6"
      >
        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-amber-500/10 text-amber-400">
          <CircleAlert size={32} />
        </div>
        <div class="space-y-2">
          <h2 class="text-2xl font-black text-slate-100">Limit Warning</h2>
          <p class="text-sm text-slate-400">
            You have <span class="text-amber-300 font-bold">{formatDuration(warningData.remainingSeconds)}</span> remaining today for this {warningData.ruleType}.
          </p>
        </div>
        <div class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono">
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

  <!-- Global countdown closing app dialog -->
  {#if showCountdownOverlay && countdownData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/90 backdrop-blur-lg"
    >
      <div
        class="silo-card max-w-md w-full p-8 border border-red-500/40 bg-slate-900/95 shadow-2xl text-center space-y-6"
      >
        <div class="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-red-500/10 text-red-500 animate-pulse">
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
        <div class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono">
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

  <!-- Global rule violation closed dialog -->
  {#if showViolationOverlay && violationData}
    <div
      transition:fade={{ duration: 200 }}
      class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md"
    >
      <div
        class="silo-card max-w-md w-full p-8 border border-red-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6"
      >
        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10 text-red-400">
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
        <div class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono">
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

  <!-- Toast Notification Container -->
  {#if toasts.length}
    <div class="fixed top-6 right-6 z-[1000] flex flex-col gap-3 max-w-sm w-full pointer-events-none">
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
