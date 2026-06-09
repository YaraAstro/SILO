<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import {
    emptyRule,
    siloApi,
    type AppSnapshot,
    type BootStatus,
    type DataUsageReport,
    type Rule,
    type Settings,
    type UsageReport,
    type UsageTimeline
  } from "$lib/siloApi";

  type ViewKey = "dashboard" | "usage" | "rules" | "focus" | "network" | "exports" | "settings";

  const views: Array<{ key: ViewKey; label: string }> = [
    { key: "dashboard", label: "Dashboard" },
    { key: "usage", label: "Usage" },
    { key: "rules", label: "Rules" },
    { key: "focus", label: "Focus" },
    { key: "network", label: "Network" },
    { key: "exports", label: "Exports" },
    { key: "settings", label: "Settings" }
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
  let loading = $state(true);
  let savingRule = $state(false);
  let savingSettings = $state(false);
  let errorMessage = $state("");
  let exportPath = $state("");

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    void loadAll();
    const timer = window.setInterval(() => void loadSnapshot(), 5000);
    void listen<AppSnapshot["activeApp"]>("update_active_app", (event) => {
      if (snapshot) snapshot = { ...snapshot, activeApp: event.payload };
    })
      .then((unlisten) => unlisteners.push(unlisten))
      .catch(() => undefined);
    void listen<{ app: string; todaySeconds: number }>("usage_update", (event) => {
      if (snapshot) snapshot = { ...snapshot, todaySeconds: event.payload.todaySeconds };
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
      const today = new Date().toISOString().slice(0, 10);
      boot = await siloApi.handshake();
      settings = boot.settings;
      await Promise.all([loadSnapshot(), loadRules(), loadUsage(today), loadDataUsage("30d")]);
    } catch (error) {
      errorMessage = toErrorMessage(error);
    } finally {
      loading = false;
    }
  }

  async function loadSnapshot() {
    try {
      snapshot = await siloApi.getAppState();
    } catch (error) {
      errorMessage = toErrorMessage(error);
    }
  }

  async function loadRules() {
    rules = await siloApi.getRules();
  }

  async function loadUsage(date: string) {
    [usage, timeline] = await Promise.all([siloApi.getUsage(date), siloApi.getUsage90d()]);
  }

  async function loadDataUsage(range: string) {
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
    const result = await siloApi.exportUsageData(dataUsage?.range ?? "30d");
    exportPath = result.filePath;
  }

  async function exportLogs() {
    const result = await siloApi.exportLogs("30d");
    exportPath = result.filePath;
  }

  function formatDuration(seconds: number) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  }

  function formatBytes(bytes: number) {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${bytes} B`;
  }

  function formatBps(bytes: number) {
    return `${formatBytes(bytes)}/s`;
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }
</script>

<svelte:head>
  <title>SILO</title>
</svelte:head>

<main class="min-h-screen bg-zinc-50 text-zinc-950">
  <div class="grid min-h-screen lg:grid-cols-[248px_1fr]">
    <aside class="border-b border-zinc-200 bg-white lg:border-b-0 lg:border-r">
      <div class="flex items-center justify-between gap-3 px-5 py-4 lg:block">
        <div>
          <p class="text-xs font-semibold uppercase text-emerald-700">Local first</p>
          <h1 class="mt-1 text-2xl font-semibold">SILO</h1>
        </div>
        <div class="rounded-md border border-zinc-200 px-2 py-1 text-xs text-zinc-600">
          v{boot?.version ?? "0.1.0"}
        </div>
      </div>

      <nav class="flex gap-1 overflow-x-auto px-3 pb-3 lg:block lg:space-y-1 lg:px-3">
        {#each views as view}
          <button
            class="min-h-10 shrink-0 rounded-md px-3 text-left text-sm font-medium outline-none transition focus-visible:ring-2 focus-visible:ring-emerald-600 lg:w-full {activeView === view.key
              ? 'bg-emerald-700 text-white'
              : 'text-zinc-700 hover:bg-zinc-100'}"
            type="button"
            onclick={() => (activeView = view.key)}
          >
            {view.label}
          </button>
        {/each}
      </nav>
    </aside>

    <section class="min-w-0">
      <header class="flex flex-col gap-3 border-b border-zinc-200 bg-white px-5 py-4 md:flex-row md:items-center md:justify-between">
        <div>
          <p class="text-sm text-zinc-500">Backend {boot?.databaseReady ? "connected" : "checking"}</p>
          <h2 class="text-xl font-semibold">{views.find((view) => view.key === activeView)?.label}</h2>
        </div>
        <button
          class="min-h-10 rounded-md bg-zinc-900 px-4 text-sm font-semibold text-white outline-none transition hover:bg-zinc-700 focus-visible:ring-2 focus-visible:ring-emerald-600 disabled:cursor-not-allowed disabled:bg-zinc-400"
          type="button"
          onclick={toggleFocus}
          disabled={loading || !!errorMessage}
        >
          {snapshot?.focusMode ? "Stop focus" : "Start focus"}
        </button>
      </header>

      <div class="px-5 py-5">
        {#if errorMessage}
          <div class="mb-4 rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800">
            {errorMessage}
          </div>
        {/if}

        {#if loading}
          <div class="rounded-md border border-zinc-200 bg-white p-6 text-sm text-zinc-600">
            Loading SILO workspace...
          </div>
        {:else if activeView === "dashboard"}
          <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <p class="text-sm text-zinc-500">Active app</p>
              <p class="mt-2 truncate text-2xl font-semibold">{snapshot?.activeApp.app}</p>
              <p class="mt-1 truncate text-sm text-zinc-500">{snapshot?.activeApp.title}</p>
              <p class="mt-1 text-xs text-zinc-400">
                PID {snapshot?.activeApp.pid ?? "n/a"} - {formatDuration(snapshot?.activeApp.elapsedSeconds ?? 0)}
              </p>
            </section>
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <p class="text-sm text-zinc-500">Today</p>
              <p class="mt-2 text-2xl font-semibold">{formatDuration(snapshot?.todaySeconds ?? 0)}</p>
              <p class="mt-1 text-sm text-zinc-500">Tracked screen time</p>
            </section>
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <p class="text-sm text-zinc-500">Rules</p>
              <p class="mt-2 text-2xl font-semibold">{snapshot?.rulesSummary.active ?? 0}</p>
              <p class="mt-1 text-sm text-zinc-500">{snapshot?.rulesSummary.total ?? 0} total configured</p>
            </section>
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <p class="text-sm text-zinc-500">Network</p>
              <p class="mt-2 text-2xl font-semibold">{formatBps(snapshot?.networkSpeed.downloadBps ?? 0)}</p>
              <p class="mt-1 text-sm text-zinc-500">Download now</p>
            </section>
          </div>

          <div class="mt-4 grid gap-4 xl:grid-cols-[1.4fr_1fr]">
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <div class="flex items-center justify-between gap-3">
                <h3 class="text-base font-semibold">Usage trend</h3>
                <span class="text-xs text-zinc-500">90 days</span>
              </div>
              {#if timeline?.days.length}
                <div class="mt-4 flex h-40 items-end gap-1">
                  {#each timeline.days.slice(-30) as day}
                    <div
                      class="min-w-2 flex-1 rounded-t bg-emerald-600"
                      title={`${day.date}: ${formatDuration(day.totalSeconds)}`}
                      style={`height: ${Math.max(6, Math.min(100, day.totalSeconds / 180))}%`}
                    ></div>
                  {/each}
                </div>
              {:else}
                <p class="mt-4 rounded-md bg-zinc-50 p-4 text-sm text-zinc-600">
                  Session history will appear after the monitoring engine starts writing sessions.
                </p>
              {/if}
            </section>

            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <h3 class="text-base font-semibold">Top apps today</h3>
              <div class="mt-3 space-y-2">
                {#if usage?.apps.length}
                  {#each usage.apps as app}
                    <div class="flex items-center justify-between gap-3 rounded-md bg-zinc-50 px-3 py-2 text-sm">
                      <span class="truncate">{app.name}</span>
                      <span class="shrink-0 font-medium">{formatDuration(app.seconds)}</span>
                    </div>
                  {/each}
                {:else}
                  <p class="rounded-md bg-zinc-50 p-4 text-sm text-zinc-600">No sessions recorded today.</p>
                {/if}
              </div>
            </section>
          </div>
        {:else if activeView === "rules"}
          <div class="grid gap-4 xl:grid-cols-[360px_1fr]">
            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <h3 class="text-base font-semibold">{ruleDraft.id ? "Edit rule" : "Add rule"}</h3>
              <div class="mt-4 space-y-4">
                <label class="block text-sm font-medium">
                  Type
                  <select class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 bg-white px-3" bind:value={ruleDraft.ruleType}>
                    <option value="app">App</option>
                    <option value="site">Site</option>
                  </select>
                </label>
                <label class="block text-sm font-medium">
                  Target
                  <input
                    class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 px-3"
                    placeholder={ruleDraft.ruleType === "app" ? "chrome.exe" : "youtube.com"}
                    bind:value={ruleDraft.target}
                  />
                </label>
                <label class="block text-sm font-medium">
                  Limit in minutes
                  <input
                    class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 px-3"
                    min="0"
                    type="number"
                    value={Math.round(ruleDraft.limitSeconds / 60)}
                    oninput={(event) =>
                      (ruleDraft.limitSeconds = Number((event.currentTarget as HTMLInputElement).value) * 60)}
                  />
                </label>
                <label class="block text-sm font-medium">
                  Enforcement
                  <select class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 bg-white px-3" bind:value={ruleDraft.enforcement}>
                    <option value="soft">Soft</option>
                    <option value="hard">Hard</option>
                    <option value="warn">Warn</option>
                  </select>
                </label>
                <label class="flex items-center gap-2 text-sm font-medium">
                  <input class="h-4 w-4 rounded border-zinc-300" type="checkbox" bind:checked={ruleDraft.active} />
                  Active
                </label>
                <div class="flex gap-2">
                  <button class="min-h-10 rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white disabled:bg-zinc-400" type="button" onclick={saveRule} disabled={savingRule}>
                    {savingRule ? "Saving" : "Save rule"}
                  </button>
                  <button class="min-h-10 rounded-md border border-zinc-300 px-4 text-sm font-semibold" type="button" onclick={() => (ruleDraft = emptyRule())}>
                    Clear
                  </button>
                </div>
              </div>
            </section>

            <section class="rounded-md border border-zinc-200 bg-white p-4">
              <h3 class="text-base font-semibold">Configured rules</h3>
              <div class="mt-3 overflow-hidden rounded-md border border-zinc-200">
                {#if rules.length}
                  {#each rules as rule}
                    <div class="grid gap-3 border-b border-zinc-200 px-3 py-3 text-sm last:border-b-0 md:grid-cols-[80px_1fr_110px_120px_120px] md:items-center">
                      <span class="rounded bg-zinc-100 px-2 py-1 text-center text-xs font-semibold uppercase">{rule.ruleType}</span>
                      <span class="min-w-0 truncate font-medium">{rule.target}</span>
                      <span>{formatDuration(rule.limitSeconds)}</span>
                      <span class={rule.active ? "text-emerald-700" : "text-zinc-500"}>{rule.active ? "Active" : "Paused"}</span>
                      <span class="flex gap-2 md:justify-end">
                        <button class="rounded-md border border-zinc-300 px-3 py-1.5 font-medium" type="button" onclick={() => editRule(rule)}>Edit</button>
                        <button class="rounded-md border border-red-200 px-3 py-1.5 font-medium text-red-700" type="button" onclick={() => removeRule(rule)}>Delete</button>
                      </span>
                    </div>
                  {/each}
                {:else}
                  <p class="p-4 text-sm text-zinc-600">No app or site rules are configured yet.</p>
                {/if}
              </div>
            </section>
          </div>
        {:else if activeView === "focus"}
          <section class="rounded-md border border-zinc-200 bg-white p-5">
            <p class="text-sm text-zinc-500">Focus mode</p>
            <h3 class="mt-2 text-3xl font-semibold">{snapshot?.focusMode ? "Running" : "Stopped"}</h3>
            <p class="mt-2 max-w-2xl text-sm text-zinc-600">
              Focus mode state is persisted in the backend runtime and broadcasts a `focus_mode_changed` event. Rule evaluation and app blocking are the next milestone.
            </p>
            <button class="mt-5 min-h-10 rounded-md bg-zinc-900 px-4 text-sm font-semibold text-white" type="button" onclick={toggleFocus}>
              {snapshot?.focusMode ? "Stop focus mode" : "Start focus mode"}
            </button>
          </section>
        {:else if activeView === "usage"}
          <section class="rounded-md border border-zinc-200 bg-white p-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <div>
                <h3 class="text-base font-semibold">Data usage</h3>
                <p class="text-sm text-zinc-500">Range: {dataUsage?.range ?? "30d"}</p>
              </div>
              <select class="min-h-10 rounded-md border border-zinc-300 bg-white px-3 text-sm" onchange={(event) => loadDataUsage((event.currentTarget as HTMLSelectElement).value)}>
                <option value="7d">7 days</option>
                <option value="30d" selected>30 days</option>
                <option value="90d">90 days</option>
              </select>
            </div>
            <div class="mt-4 grid gap-4 md:grid-cols-2">
              {@render Metric("Upload", formatBytes(dataUsage?.totalUploadBytes ?? 0))}
              {@render Metric("Download", formatBytes(dataUsage?.totalDownloadBytes ?? 0))}
            </div>
            {@render UsageTable("Applications", dataUsage?.apps ?? [])}
            {@render UsageTable("Sites", dataUsage?.sites ?? [])}
          </section>
        {:else if activeView === "network"}
          <section class="rounded-md border border-zinc-200 bg-white p-5">
            <h3 class="text-base font-semibold">Live network speed</h3>
            <div class="mt-4 grid gap-4 md:grid-cols-2">
              {@render Metric("Upload", formatBps(snapshot?.networkSpeed.uploadBps ?? 0))}
              {@render Metric("Download", formatBps(snapshot?.networkSpeed.downloadBps ?? 0))}
            </div>
            <p class="mt-4 text-sm text-zinc-600">
              The command and storage table are ready. Windows network sampling is intentionally left for the network monitoring phase.
            </p>
          </section>
        {:else if activeView === "exports"}
          <section class="rounded-md border border-zinc-200 bg-white p-5">
            <h3 class="text-base font-semibold">Exports and backup</h3>
            <p class="mt-2 text-sm text-zinc-600">Exports are written to the SILO local app data directory.</p>
            <div class="mt-4 flex flex-wrap gap-2">
              <button class="min-h-10 rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white" type="button" onclick={exportUsage}>Export usage</button>
              <button class="min-h-10 rounded-md border border-zinc-300 px-4 text-sm font-semibold" type="button" onclick={exportLogs}>Export logs</button>
              <button class="min-h-10 rounded-md border border-zinc-300 px-4 text-sm font-semibold" type="button" onclick={completeBackup}>Mark backup complete</button>
            </div>
            {#if settings?.lastBackupAt}
              <p class="mt-4 text-sm text-zinc-600">Last backup: {settings.lastBackupAt}</p>
            {/if}
            {#if exportPath}
              <p class="mt-4 break-all rounded-md bg-zinc-50 p-3 text-sm text-zinc-700">{exportPath}</p>
            {/if}
          </section>
        {:else if activeView === "settings" && settings}
          <section class="rounded-md border border-zinc-200 bg-white p-5">
            <h3 class="text-base font-semibold">Settings</h3>
            <div class="mt-4 grid gap-4 md:grid-cols-2">
              <label class="block text-sm font-medium">
                Theme
                <select class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 bg-white px-3" bind:value={settings.theme}>
                  <option value="system">System</option>
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                </select>
              </label>
              <label class="block text-sm font-medium">
                Retention days
                <input class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 px-3" min="1" type="number" bind:value={settings.retentionDays} />
              </label>
              <label class="block text-sm font-medium">
                Network sample interval
                <input class="mt-1 min-h-10 w-full rounded-md border border-zinc-300 px-3" min="1" type="number" bind:value={settings.sampleIntervalSeconds} />
              </label>
              <div class="space-y-3 pt-6">
                <label class="flex items-center gap-2 text-sm font-medium">
                  <input class="h-4 w-4 rounded border-zinc-300" type="checkbox" bind:checked={settings.autoStart} />
                  Start SILO with Windows
                </label>
                <label class="flex items-center gap-2 text-sm font-medium">
                  <input class="h-4 w-4 rounded border-zinc-300" type="checkbox" bind:checked={settings.notificationsEnabled} />
                  Enable notifications
                </label>
              </div>
            </div>
            <button class="mt-5 min-h-10 rounded-md bg-emerald-700 px-4 text-sm font-semibold text-white disabled:bg-zinc-400" type="button" onclick={saveSettings} disabled={savingSettings}>
              {savingSettings ? "Saving" : "Save settings"}
            </button>
          </section>
        {/if}
      </div>
    </section>
  </div>
</main>

{#snippet Metric(label: string, value: string)}
  <div class="rounded-md border border-zinc-200 bg-zinc-50 p-4">
    <p class="text-sm text-zinc-500">{label}</p>
    <p class="mt-1 text-2xl font-semibold">{value}</p>
  </div>
{/snippet}

{#snippet UsageTable(title: string, rows: Array<{ name: string; uploadBytes: number; downloadBytes: number }>)}
  <div class="mt-5">
    <h4 class="text-sm font-semibold">{title}</h4>
    <div class="mt-2 overflow-hidden rounded-md border border-zinc-200">
      {#if rows.length}
        {#each rows as row}
          <div class="grid gap-2 border-b border-zinc-200 px-3 py-2 text-sm last:border-b-0 md:grid-cols-[1fr_120px_120px]">
            <span class="truncate font-medium">{row.name}</span>
            <span class="text-zinc-600 md:text-right">{formatBytes(row.uploadBytes)}</span>
            <span class="text-zinc-600 md:text-right">{formatBytes(row.downloadBytes)}</span>
          </div>
        {/each}
      {:else}
        <p class="p-4 text-sm text-zinc-600">No usage records yet.</p>
      {/if}
    </div>
  </div>
{/snippet}
