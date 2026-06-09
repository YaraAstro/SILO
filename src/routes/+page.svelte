<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
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
    type UsageTimeline
  } from "$lib/siloApi";

  type ViewKey = "dashboard" | "rules" | "stats" | "network" | "settings";
  type RangeKey = "7d" | "30d" | "90d";

  const navItems = [
    { key: "dashboard", label: "Dashboard", icon: House },
    { key: "rules", label: "Rules", icon: Shield },
    { key: "stats", label: "Stats", icon: ChartColumn },
    { key: "network", label: "Network", icon: Wifi },
    { key: "settings", label: "Settings", icon: SettingsIcon }
  ];

  let activeView = $state<ViewKey>("dashboard");
  let boot = $state<BootStatus | null>(null);
  let snapshot = $state<AppSnapshot | null>(null);
  let rules = $state<Rule[]>([]);
  let settings = $state<Settings | null>(null);
  let usage = $state<UsageReport | null>(null);
  let timeline = $state<UsageTimeline | null>(null);
  let dataUsage = $state<DataUsageReport | null>(null);
  let ruleDraft = $state<Rule>(emptyRule());
  let dataRange = $state<RangeKey>("30d");
  let ruleSearch = $state("");
  let loading = $state(true);
  let savingRule = $state(false);
  let savingSettings = $state(false);
  let exporting = $state(false);
  let errorMessage = $state("");
  let exportPath = $state("");

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    void loadAll();
    const timer = window.setInterval(() => void refreshLiveState(), 5000);

    void listen<AppSnapshot["activeApp"]>("update_active_app", (event) => {
      if (snapshot) snapshot = { ...snapshot, activeApp: event.payload };
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{ app: string; todaySeconds: number }>("usage_update", (event) => {
      if (snapshot) snapshot = { ...snapshot, todaySeconds: event.payload.todaySeconds };
      void loadTodayUsage();
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);

    void listen<{ enabled: boolean }>("focus_mode_changed", (event) => {
      if (snapshot) snapshot = { ...snapshot, focusMode: event.payload.enabled };
      if (boot) boot = { ...boot, focusMode: event.payload.enabled };
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
    errorMessage = "";
    try {
      boot = await siloApi.handshake();
      settings = boot.settings;
      await Promise.all([loadSnapshot(), loadRules(), loadTodayUsage(), loadTimeline(), loadDataUsage(dataRange)]);
    } catch (error) {
      errorMessage = toErrorMessage(error);
    } finally {
      loading = false;
    }
  }

  async function refreshLiveState() {
    await Promise.all([loadSnapshot(), loadDataUsage(dataRange)]).catch((error) => {
      errorMessage = toErrorMessage(error);
    });
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

  async function saveRule() {
    if (!ruleDraft.target.trim()) {
      errorMessage = "Rule target is required.";
      return;
    }

    savingRule = true;
    errorMessage = "";
    try {
      await siloApi.saveRule({ ...ruleDraft, target: ruleDraft.target.trim() });
      ruleDraft = emptyRule();
      await Promise.all([loadRules(), loadSnapshot()]);
    } catch (error) {
      errorMessage = toErrorMessage(error);
    } finally {
      savingRule = false;
    }
  }

  async function removeRule(rule: Rule) {
    if (rule.id === null || !window.confirm(`Delete rule for ${rule.target}?`)) return;
    await siloApi.deleteRule(rule.id);
    await Promise.all([loadRules(), loadSnapshot()]);
  }

  function editRule(rule: Rule) {
    ruleDraft = { ...rule };
    activeView = "rules";
  }

  async function toggleFocus() {
    const enabled = await siloApi.toggleFocusMode();
    if (snapshot) snapshot = { ...snapshot, focusMode: enabled };
    if (boot) boot = { ...boot, focusMode: enabled };
  }

  async function saveSettings() {
    if (!settings) return;
    savingSettings = true;
    errorMessage = "";
    try {
      settings = await siloApi.saveSettings(settings);
      if (boot) boot = { ...boot, settings };
    } catch (error) {
      errorMessage = toErrorMessage(error);
    } finally {
      savingSettings = false;
    }
  }

  async function completeBackup() {
    settings = await siloApi.markBackupComplete();
  }

  async function exportUsage() {
    exporting = true;
    errorMessage = "";
    try {
      const result = await siloApi.exportUsageData(dataRange);
      exportPath = result.filePath;
    } catch (error) {
      errorMessage = toErrorMessage(error);
    } finally {
      exporting = false;
    }
  }

  async function exportLogs() {
    exporting = true;
    errorMessage = "";
    try {
      const result = await siloApi.exportLogs(dataRange);
      exportPath = result.filePath;
    } catch (error) {
      errorMessage = toErrorMessage(error);
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

  function formatClock(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    const secs = Math.floor(safeSeconds % 60);
    if (hours > 0) return `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
    return `${minutes}:${String(secs).padStart(2, "0")}`;
  }

  function formatBytes(bytes: number) {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${bytes} B`;
  }

  function formatBps(bytes: number) {
    return `${formatBytes(bytes)}/s`;
  }

  function formatDateLabel(date: string) {
    return new Intl.DateTimeFormat(undefined, { weekday: "short" }).format(new Date(`${date}T00:00:00`));
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function filteredRules() {
    const query = ruleSearch.trim().toLowerCase();
    if (!query) return rules;
    return rules.filter((rule) => rule.target.toLowerCase().includes(query) || rule.ruleType.includes(query));
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
        borderRadius: 6
      }
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
        borderRadius: 6
      },
      {
        label: "Upload",
        data: [Math.round(uploadTotal / 1_048_576)],
        backgroundColor: "rgba(139, 92, 246, 0.85)",
        borderColor: "#a78bfa",
        borderRadius: 6
      }
    ];
  }

  function totalTrackedSeconds() {
    return timeline?.days.reduce((sum, day) => sum + day.totalSeconds, 0) ?? 0;
  }

  function averageTrackedSeconds() {
    const days = timeline?.days.filter((day) => day.totalSeconds > 0) ?? [];
    if (!days.length) return 0;
    return Math.round(days.reduce((sum, day) => sum + day.totalSeconds, 0) / days.length);
  }

  function bestTrackedDay() {
    const days = timeline?.days ?? [];
    return days.reduce((best, day) => (day.totalSeconds > best.totalSeconds ? day : best), { date: "", totalSeconds: 0 });
  }

  function totalDataBytes() {
    return (dataUsage?.totalDownloadBytes ?? 0) + (dataUsage?.totalUploadBytes ?? 0);
  }

  function consumerTotal(consumer: DataConsumer) {
    return consumer.downloadBytes + consumer.uploadBytes;
  }
</script>

<svelte:head>
  <title>SILO</title>
</svelte:head>

<main class="min-h-screen bg-silo text-slate-100">
  <div class="mx-auto min-h-screen w-full max-w-7xl px-4 pb-28 pt-8 sm:px-6 lg:px-8">
    {#if errorMessage}
      <div class="mb-5 flex items-start gap-3 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-100">
        <CircleAlert class="mt-0.5 shrink-0 text-red-300" size={18} />
        <p>{errorMessage}</p>
      </div>
    {/if}

    {#if loading}
      <section class="flex min-h-[70vh] items-center justify-center">
        <div class="text-center">
          <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-teal-400/15 text-teal-300">
            <Database size={26} />
          </div>
          <p class="mt-4 text-lg font-bold">Loading SILO workspace</p>
          <p class="mt-1 text-sm text-slate-500">Connecting to local monitoring services.</p>
        </div>
      </section>
    {:else if activeView === "dashboard"}
      <section class="space-y-6">
        <header class="text-center">
          <h1 class="text-5xl font-black tracking-normal">
            <span class="text-teal-300">SI</span><span class="text-blue-400">L</span><span class="text-violet-400">O</span>
          </h1>
          <p class="mt-3 text-base text-slate-400">Your focus space</p>
          <button class="silo-button mt-7" type="button" onclick={toggleFocus}>
            <Power size={17} />
            {snapshot?.focusMode ? "Stop Focus" : "Start Focus"}
          </button>
        </header>

        <div class="grid gap-5 md:grid-cols-3">
          <section class="silo-card p-6">
            <div class="flex items-center gap-3">
              <IconBadge icon={Monitor} tone="teal" label="Active application" />
              <h2 class="text-lg font-bold">Active Application</h2>
            </div>
            <p class="mt-6 truncate text-2xl font-bold">{snapshot?.activeApp.app || "No active app"}</p>
            <p class="mt-1 truncate text-sm text-slate-500">{snapshot?.activeApp.title || "Current window"}</p>
            <p class="mt-5 flex items-center gap-2 font-mono text-lg font-bold text-teal-300">
              <Clock size={18} />
              {formatClock(snapshot?.activeApp.elapsedSeconds ?? 0)}
            </p>
          </section>

          <section class="silo-card p-6">
            <div class="flex items-center gap-3">
              <IconBadge icon={Timer} tone="purple" label="Session progress" />
              <h2 class="text-lg font-bold">Session Progress</h2>
            </div>
            <p class="mt-6 text-3xl font-black">{formatDuration(snapshot?.todaySeconds ?? 0)}</p>
            <p class="mt-1 text-sm text-slate-500">Tracked today</p>
            <div class="mt-5 h-2 rounded-full bg-slate-800">
              <div class="h-2 rounded-full bg-teal-400" style={`width: ${Math.min(100, ((snapshot?.todaySeconds ?? 0) / 28800) * 100)}%`}></div>
            </div>
            <p class="mt-4 text-sm text-slate-400">{snapshot?.focusMode ? "Focus mode is running" : "Focus mode is stopped"}</p>
          </section>

          <section class="silo-card p-6">
            <div class="flex items-center gap-3">
              <IconBadge icon={Target} tone="teal" label="Focus score" />
              <h2 class="text-lg font-bold">Focus Score</h2>
            </div>
            <EmptyState
              compact
              icon={Gauge}
              title="Score pending"
              message="Productivity scoring needs rule violations and focus-session history from later backend phases."
            />
          </section>
        </div>

        <div class="grid gap-5 xl:grid-cols-3">
          <section class="silo-card p-6 xl:col-span-2">
            <div class="flex items-center justify-between gap-4">
              <div class="flex items-center gap-3">
                <IconBadge icon={Activity} tone="teal" label="Focus pattern" />
                <div>
                  <h2 class="text-lg font-bold">Today's Focus Pattern</h2>
                  <p class="text-sm text-slate-500">Screen-time trend from stored sessions</p>
                </div>
              </div>
            </div>
            {#if weekDays().length}
              <div class="mt-6">
                <TrendChart labels={chartLabels()} datasets={focusChartData()} height={250} />
              </div>
            {:else}
              <EmptyState
                icon={ChartColumn}
                title="No trend data yet"
                message="Session history will appear here after SILO records foreground app activity."
              />
            {/if}
          </section>

          <section class="silo-card p-6">
            <div class="flex items-center justify-between gap-3">
              <div class="flex items-center gap-3">
                <IconBadge icon={Wifi} tone="teal" label="Network" />
                <h2 class="text-lg font-bold">Network</h2>
              </div>
              <span class="flex items-center gap-1 text-xs text-emerald-300">
                <span class="h-2 w-2 rounded-full bg-emerald-400"></span>
                Live
              </span>
            </div>
            <div class="mt-6 grid gap-4 sm:grid-cols-2 xl:grid-cols-1">
              <div>
                <p class="flex items-center gap-2 text-sm text-slate-500"><Download size={15} /> Download</p>
                <p class="mt-1 text-2xl font-black">{formatBps(snapshot?.networkSpeed.downloadBps ?? 0)}</p>
              </div>
              <div>
                <p class="flex items-center gap-2 text-sm text-slate-500"><Upload size={15} /> Upload</p>
                <p class="mt-1 text-2xl font-black">{formatBps(snapshot?.networkSpeed.uploadBps ?? 0)}</p>
              </div>
            </div>
            <p class="mt-6 rounded-lg border border-slate-700 bg-slate-950/35 p-3 text-sm text-slate-500">
              OS network sampling is not active yet, so speeds may remain at zero.
            </p>
          </section>
        </div>

        <div class="grid gap-5 xl:grid-cols-[1.35fr_0.65fr]">
          <section class="silo-card p-6">
            <div class="flex items-center gap-3">
              <IconBadge icon={Sparkles} tone="purple" label="AI insights" />
              <div>
                <h2 class="text-lg font-bold">AI Insights</h2>
                <p class="text-sm text-slate-500">Personalized productivity tips</p>
              </div>
            </div>
            <div class="mt-5 grid gap-3">
              <EmptyState
                compact
                icon={Sparkles}
                title="Insights unavailable"
                message="AI productivity insights are planned after richer focus, rules, and usage signals are stored."
              />
            </div>
          </section>

          <section class="silo-card p-6">
            <div class="flex items-center gap-3">
              <IconBadge icon={ChartColumn} tone="teal" label="Usage" />
              <h2 class="text-lg font-bold">Today's Usage</h2>
            </div>
            <div class="mt-5 space-y-4">
              {#if usage?.apps.length}
                {#each usage.apps.slice(0, 6) as app}
                  <div>
                    <div class="flex items-center justify-between gap-3 text-sm">
                      <span class="truncate font-semibold">{app.name}</span>
                      <span class="shrink-0 text-slate-400">{formatDuration(app.seconds)}</span>
                    </div>
                    <div class="mt-2 h-1.5 rounded-full bg-slate-800">
                      <div
                        class="h-1.5 rounded-full bg-teal-400"
                        style={`width: ${Math.max(4, Math.min(100, (app.seconds / Math.max(1, usage.totalSeconds)) * 100))}%`}
                      ></div>
                    </div>
                  </div>
                {/each}
              {:else}
                <EmptyState compact icon={Monitor} title="No usage yet" message="Tracked applications will appear here." />
              {/if}
            </div>
            <div class="mt-6 border-t border-slate-800 pt-4 text-right text-xl font-black text-teal-300">
              {formatDuration(usage?.totalSeconds ?? 0)}
            </div>
          </section>
        </div>
      </section>
    {:else if activeView === "rules"}
      <section class="mx-auto max-w-5xl space-y-6">
        <header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h1 class="text-4xl font-black">Rules &amp; Limits</h1>
            <p class="mt-2 text-slate-400">Manage your app and website restrictions</p>
          </div>
          <button class="silo-button" type="button" onclick={() => (ruleDraft = emptyRule())}>
            <Plus size={18} />
            Add Rule
          </button>
        </header>

        <section class="silo-card p-5">
          <div class="grid gap-4 lg:grid-cols-[1fr_170px_150px_110px]">
            <label class="text-sm font-semibold text-slate-300">
              Target
              <input
                class="silo-input mt-2"
                placeholder={ruleDraft.ruleType === "app" ? "chrome.exe" : "youtube.com"}
                bind:value={ruleDraft.target}
              />
            </label>
            <label class="text-sm font-semibold text-slate-300">
              Type
              <select class="silo-input mt-2" bind:value={ruleDraft.ruleType}>
                <option value="app">App</option>
                <option value="site">Site</option>
              </select>
            </label>
            <label class="text-sm font-semibold text-slate-300">
              Enforcement
              <select class="silo-input mt-2" bind:value={ruleDraft.enforcement}>
                <option value="soft">Soft Warning</option>
                <option value="hard">Hard Block</option>
                <option value="warn">Warning</option>
              </select>
            </label>
            <label class="text-sm font-semibold text-slate-300">
              Minutes
              <input
                class="silo-input mt-2"
                min="0"
                type="number"
                value={Math.round(ruleDraft.limitSeconds / 60)}
                oninput={(event) => (ruleDraft.limitSeconds = Number((event.currentTarget as HTMLInputElement).value) * 60)}
              />
            </label>
          </div>
          <div class="mt-4 flex flex-wrap items-center justify-between gap-3">
            <label class="inline-flex items-center gap-3 text-sm font-semibold text-slate-300">
              <input class="h-4 w-4 rounded border-slate-600 bg-slate-950 accent-teal-400" type="checkbox" bind:checked={ruleDraft.active} />
              Active rule
            </label>
            <div class="flex gap-2">
              <button class="silo-button-secondary" type="button" onclick={() => (ruleDraft = emptyRule())}>Clear</button>
              <button class="silo-button" type="button" onclick={saveRule} disabled={savingRule}>
                {savingRule ? "Saving" : ruleDraft.id ? "Save Rule" : "Add Rule"}
              </button>
            </div>
          </div>
        </section>

        <div class="relative max-w-md">
          <Search class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={17} />
          <input class="silo-input pl-10" placeholder="Filter rules..." bind:value={ruleSearch} />
        </div>

        <section class="space-y-4">
          {#if filteredRules().length}
            {#each filteredRules() as rule}
              <article class="silo-card flex items-center gap-4 p-5">
                <IconBadge icon={rule.ruleType === "site" ? Globe : Monitor} tone={rule.ruleType === "site" ? "teal" : "purple"} label={rule.ruleType} size="lg" />
                <div class="min-w-0 flex-1">
                  <h2 class="truncate text-xl font-bold">{rule.target}</h2>
                  <div class="mt-2 flex flex-wrap items-center gap-2 text-sm text-slate-400">
                    <span class="flex items-center gap-1"><Clock size={14} /> {formatDuration(rule.limitSeconds)} daily</span>
                    <span class="rounded-md px-2 py-1 text-xs font-semibold {rule.enforcement === 'hard' ? 'bg-red-500/15 text-red-300' : rule.enforcement === 'soft' ? 'bg-amber-500/15 text-amber-300' : 'bg-blue-500/15 text-blue-300'}">
                      {rule.enforcement === "hard" ? "Hard Block" : rule.enforcement === "soft" ? "Soft Warning" : "Warning"}
                    </span>
                    <span class={rule.active ? "text-emerald-300" : "text-slate-500"}>{rule.active ? "Active" : "Paused"}</span>
                  </div>
                </div>
                <button class="silo-icon-button" type="button" aria-label={`Edit ${rule.target}`} onclick={() => editRule(rule)}>
                  <Info size={18} />
                </button>
                <button class="silo-icon-button text-red-300 hover:bg-red-500/10" type="button" aria-label={`Delete ${rule.target}`} onclick={() => removeRule(rule)}>
                  <Trash2 size={18} />
                </button>
              </article>
            {/each}
          {:else}
            <EmptyState icon={Shield} title="No rules found" message="Add an app or website rule to start managing distractions." />
          {/if}
        </section>
      </section>
    {:else if activeView === "stats"}
      <section class="mx-auto max-w-6xl space-y-6">
        <header>
          <h1 class="text-4xl font-black">Statistics</h1>
          <p class="mt-2 text-slate-400">Your productivity insights</p>
        </header>

        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          <MetricCard icon={Clock} title="Total Focus Time" value={formatDuration(totalTrackedSeconds())} trend={totalTrackedSeconds() ? "From stored history" : ""} />
          <MetricCard icon={Target} tone="purple" title="Daily Average" value={formatDuration(averageTrackedSeconds())} trend={averageTrackedSeconds() ? "Active days only" : ""} />
          <MetricCard icon={Flame} tone="yellow" title="Streak" value="Unavailable" caption="Needs focus-session goals" />
          <MetricCard icon={Award} tone="yellow" title="Best Day" value={bestTrackedDay().date ? formatDateLabel(bestTrackedDay().date) : "None"} caption={bestTrackedDay().totalSeconds ? formatDuration(bestTrackedDay().totalSeconds) : "No history yet"} />
          <MetricCard icon={Calendar} tone="blue" title="This Week" value={formatDuration(weekDays().reduce((sum, day) => sum + day.totalSeconds, 0))} trend={weekDays().length ? "Last 7 days" : ""} />
        </div>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Calendar} tone="teal" label="Activity heatmap" />
            <div>
              <h2 class="text-lg font-bold">Activity Heatmap</h2>
              <p class="text-sm text-slate-500">Last 12 weeks</p>
            </div>
          </div>
          {#if timeline?.days.length}
            <div class="mt-6 grid max-w-xl grid-flow-col grid-rows-7 gap-2">
              {#each timeline.days.slice(-84) as day}
                <span
                  class="h-3 w-3 rounded-full {day.totalSeconds > 14400
                    ? 'bg-teal-300'
                    : day.totalSeconds > 7200
                      ? 'bg-teal-500'
                      : day.totalSeconds > 0
                        ? 'bg-teal-900'
                        : 'bg-slate-900'}"
                  title={`${day.date}: ${formatDuration(day.totalSeconds)}`}
                ></span>
              {/each}
            </div>
            <div class="mt-5 flex max-w-xl items-center justify-between text-xs text-slate-500">
              <span>Less</span>
              <span>More</span>
            </div>
          {:else}
            <EmptyState icon={Calendar} title="No heatmap data" message="Daily activity dots will appear once sessions are recorded." />
          {/if}
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center justify-between gap-4">
            <h2 class="text-lg font-bold">Time Analysis</h2>
            <span class="rounded-lg bg-slate-800 px-3 py-2 text-xs font-bold text-slate-300">Week</span>
          </div>
          {#if weekDays().length}
            <div class="mt-6">
              <TrendChart labels={chartLabels()} datasets={focusChartData()} height={330} />
            </div>
          {:else}
            <EmptyState icon={ChartColumn} title="No time analysis yet" message="The weekly chart will populate from usage history." />
          {/if}
        </section>

        <section class="silo-card p-6">
          <h2 class="text-lg font-bold">Most Used Apps</h2>
          <div class="mt-5 space-y-4">
            {#if usage?.apps.length}
              {#each usage.apps.slice(0, 6) as app}
                <div>
                  <div class="flex items-center justify-between gap-3">
                    <span class="truncate font-semibold">{app.name}</span>
                    <span class="text-sm text-slate-400">{formatDuration(app.seconds)}</span>
                  </div>
                  <div class="mt-2 h-2 rounded-full bg-slate-800">
                    <div class="h-2 rounded-full bg-blue-400" style={`width: ${Math.max(4, Math.min(100, (app.seconds / Math.max(1, usage.totalSeconds)) * 100))}%`}></div>
                  </div>
                </div>
              {/each}
            {:else}
              <EmptyState compact icon={Monitor} title="No app usage today" message="Applications will appear after SILO tracks them." />
            {/if}
          </div>
        </section>
      </section>
    {:else if activeView === "network"}
      <section class="mx-auto max-w-6xl space-y-6">
        <header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <div class="flex items-center gap-3">
              <Wifi class="text-teal-300" size={28} />
              <h1 class="text-4xl font-black">Network</h1>
            </div>
            <p class="mt-2 text-slate-400">Live speed, per-app and per-site bandwidth usage</p>
          </div>
          <button class="silo-button-secondary" type="button" onclick={exportUsage} disabled={exporting}>
            <FileDown size={16} />
            Export Usage
          </button>
        </header>

        <section class="silo-card p-6">
          <div class="flex items-center justify-between text-sm text-slate-500">
            <span class="flex items-center gap-2"><span class="h-2 w-2 rounded-full bg-emerald-400"></span> Live network speed</span>
            <span>Sample interval: {settings?.sampleIntervalSeconds ?? 5}s</span>
          </div>
          <div class="mt-8 grid gap-8 md:grid-cols-2">
            <div>
              <p class="flex items-center gap-2 text-sm text-slate-500"><Download size={16} /> Download</p>
              <p class="mt-2 text-5xl font-black text-teal-300">{formatBps(snapshot?.networkSpeed.downloadBps ?? 0)}</p>
              <p class="mt-2 text-sm text-slate-500">Today: {formatBytes(dataUsage?.totalDownloadBytes ?? 0)}</p>
            </div>
            <div>
              <p class="flex items-center gap-2 text-sm text-slate-500"><Upload size={16} /> Upload</p>
              <p class="mt-2 text-5xl font-black text-violet-400">{formatBps(snapshot?.networkSpeed.uploadBps ?? 0)}</p>
              <p class="mt-2 text-sm text-slate-500">Today: {formatBytes(dataUsage?.totalUploadBytes ?? 0)}</p>
            </div>
          </div>
          <div class="mt-8 h-1 rounded-full bg-teal-400/70"></div>
        </section>

        <section class="silo-card p-6">
          <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <h2 class="text-lg font-bold">Usage breakdown</h2>
              <p class="text-sm text-slate-500">See how much data each app and site used over the selected period</p>
            </div>
            <div class="flex rounded-lg bg-slate-800 p-1">
              {#each ["7d", "30d", "90d"] as range}
                <button
                  class="rounded-md px-3 py-2 text-xs font-bold transition {dataRange === range ? 'bg-slate-950 text-slate-100' : 'text-slate-400 hover:text-slate-100'}"
                  type="button"
                  onclick={() => loadDataUsage(range as RangeKey)}
                >
                  {range}
                </button>
              {/each}
            </div>
          </div>
          <div class="mt-6 grid gap-5 sm:grid-cols-4">
            <div><p class="text-sm text-slate-500">Total</p><p class="mt-1 text-2xl font-black">{formatBytes(totalDataBytes())}</p></div>
            <div><p class="text-sm text-slate-500">Download</p><p class="mt-1 text-2xl font-black">{formatBytes(dataUsage?.totalDownloadBytes ?? 0)}</p></div>
            <div><p class="text-sm text-slate-500">Upload</p><p class="mt-1 text-2xl font-black">{formatBytes(dataUsage?.totalUploadBytes ?? 0)}</p></div>
            <div><p class="text-sm text-slate-500">Range</p><p class="mt-1 text-2xl font-black">{dataRange}</p></div>
          </div>
          {#if totalDataBytes() > 0}
            <div class="mt-6">
              <TrendChart labels={["Selected range"]} datasets={networkChartData()} height={220} />
            </div>
          {/if}
        </section>

        <div class="grid gap-5 lg:grid-cols-2">
          {@render ConsumerList("Top Apps", dataUsage?.apps ?? [], Monitor)}
          {@render ConsumerList("Top Sites", dataUsage?.sites ?? [], Globe)}
        </div>

        <section class="silo-card p-6">
          <h2 class="text-lg font-bold">Usage History</h2>
          <EmptyState
            icon={HardDrive}
            title="Network history unavailable"
            message="Historical speed samples will appear after the network monitoring backend starts persisting samples."
          />
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
            {@render SettingToggle("Launch at startup", "Start SILO when you log in", settings.autoStart, (checked) => (settings = { ...settings!, autoStart: checked }))}
            {@render SettingToggle("Notifications", "Get time limit warnings and summaries", settings.notificationsEnabled, (checked) => (settings = { ...settings!, notificationsEnabled: checked }))}
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
              <input class="silo-input mt-2" min="1" type="number" bind:value={settings.retentionDays} />
            </label>
            <label class="text-sm font-semibold text-slate-300">
              Network polling interval
              <input class="silo-input mt-2" min="1" type="number" bind:value={settings.sampleIntervalSeconds} />
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
            <p class="mt-3 text-sm text-slate-500">The UI uses the dark SILO shell in this implementation pass.</p>
          </div>
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Database} tone="teal" label="Privacy and data" />
            <h2 class="text-lg font-bold">Privacy &amp; Data</h2>
          </div>
          <div class="mt-6 space-y-3">
            <button class="silo-button-secondary w-full justify-center" type="button" onclick={exportUsage} disabled={exporting}>
              <Download size={16} />
              Export Usage Data
            </button>
            <button class="silo-button-secondary w-full justify-center" type="button" onclick={exportLogs} disabled={exporting}>
              <FileDown size={16} />
              Export Logs
            </button>
            <button class="silo-button-secondary w-full justify-center" type="button" onclick={completeBackup}>
              <RotateCcw size={16} />
              Mark Backup Complete
            </button>
          </div>
          {#if settings.lastBackupAt}
            <p class="mt-4 text-sm text-slate-500">Last backup: {settings.lastBackupAt}</p>
          {/if}
          {#if exportPath}
            <p class="mt-4 break-all rounded-lg bg-slate-950/60 p-3 text-sm text-slate-400">{exportPath}</p>
          {/if}
        </section>

        <section class="silo-card p-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Keyboard} tone="neutral" label="Keyboard shortcuts" />
            <h2 class="text-lg font-bold">Keyboard Shortcuts</h2>
          </div>
          <EmptyState
            compact
            icon={Keyboard}
            title="Shortcut registration pending"
            message="Global hotkeys are part of the later tray and system integration phase."
          />
        </section>

        <section class="flex justify-end">
          <button class="silo-button" type="button" onclick={saveSettings} disabled={savingSettings}>
            {savingSettings ? "Saving" : "Save Settings"}
          </button>
        </section>
      </section>
    {/if}
  </div>

  <BottomNav items={navItems} active={activeView} onSelect={(key) => (activeView = key as ViewKey)} />
</main>

{#snippet ConsumerList(title: string, rows: DataConsumer[], icon: any)}
  <section class="silo-card p-6">
    <div class="flex items-center justify-between gap-3">
      <div class="flex items-center gap-3">
        <IconBadge {icon} tone="teal" label={title} />
        <h2 class="text-lg font-bold">{title}</h2>
      </div>
      <span class="text-xs text-slate-500">{dataRange}</span>
    </div>
    <div class="mt-6 space-y-5">
      {#if rows.length}
        {#each rows.slice(0, 8) as row}
          <div>
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate font-bold">{row.name}</p>
                <p class="text-sm text-slate-500">
                  Down {formatBytes(row.downloadBytes)} · Up {formatBytes(row.uploadBytes)}
                </p>
              </div>
              <p class="shrink-0 text-sm font-bold">{formatBytes(consumerTotal(row))}</p>
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
        <EmptyState compact icon={HardDrive} title="No usage records" message="Per-app and per-site data usage will appear after attribution is implemented." />
      {/if}
    </div>
  </section>
{/snippet}

{#snippet SettingToggle(title: string, description: string, checked: boolean, onChange: (checked: boolean) => void)}
  <div class="flex items-center justify-between gap-4">
    <div>
      <p class="font-semibold">{title}</p>
      <p class="mt-1 text-sm text-slate-500">{description}</p>
    </div>
    <ToggleSwitch {checked} label={title} onchange={onChange} />
  </div>
{/snippet}
