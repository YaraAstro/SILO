<script lang="ts">
  import { onMount } from "svelte";
  import { fade, scale } from "svelte/transition";
  import {
    ChartColumn,
    Clock,
    Monitor,
    Globe,
    Award,
    Search,
    Plus,
    Wifi,
    Download,
    Upload,
    Flame,
    Calendar,
    Target,
  } from "lucide-svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import MetricCard from "$lib/components/MetricCard.svelte";
  import TrendChart from "$lib/components/TrendChart.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import { cleanDomain } from "$lib/utils/cleanDomain";
  import { siloApi, emptyRule } from "$lib/siloApi";
  import type { Rule, UsageReport, DataUsageReport, UsageDayBytes, UsageTimeline, RuleStats } from "$lib/siloApi";

  let {
    rules,
    ruleDraft = $bindable(),
    showRuleForm = $bindable(),
    activeView = $bindable(),
  } = $props<{
    rules: Rule[];
    ruleDraft: Rule;
    showRuleForm: boolean;
    activeView: string;
  }>();

  type RangeKey = "7d" | "30d" | "90d";

  // Stats specific state variables
  let statsSubTab = $state<"screentime" | "network" | "habits">("habits");
  let statsRange = $state<"today" | RangeKey>("7d");
  let statsScreenTab = $state<"apps" | "sites">("apps");
  let statsNetworkTab = $state<"apps" | "sites">("apps");
  let statsSearch = $state("");
  let screenTimePage = $state(1);
  let networkPage = $state(1);
  let heatmapTab = $state<"screentime" | "network">("screentime");

  // Loaded database entities
  let statsUsage = $state<UsageReport | null>(null);
  let statsDataUsage = $state<DataUsageReport | null>(null);
  let statsNetworkHistory = $state<UsageDayBytes[]>([]);
  let heatmapNetworkHistory = $state<UsageDayBytes[]>([]);
  let statsRuleStats = $state<RuleStats[]>([]);
  let timeline = $state<UsageTimeline | null>(null);

  const pageSize = 5;

  // Svelte 5 effects for pagination reset and data hydration
  $effect(() => {
    statsSearch;
    statsScreenTab;
    statsNetworkTab;
    statsSubTab;
    statsRange;
    screenTimePage = 1;
    networkPage = 1;
  });

  async function loadStatsData(range: "today" | RangeKey) {
    try {
      const [uReport, dReport, netHist, rStats] = await Promise.all([
        siloApi.getUsageRange(range),
        siloApi.getDataUsage(range),
        siloApi.getNetworkHistory(range === "90d" ? "90d" : range === "7d" ? "7d" : "30d"),
        siloApi.getRuleStats(range)
      ]);
      statsUsage = uReport;
      statsDataUsage = dReport;
      statsNetworkHistory = netHist;
      statsRuleStats = rStats;
    } catch (error) {
      console.error("Failed to load statistics range data:", error);
    }
  }

  $effect(() => {
    if (statsRange) {
      void loadStatsData(statsRange);
    }
  });

  onMount(async () => {
    try {
      timeline = await siloApi.getUsage90d();
      heatmapNetworkHistory = await siloApi.getNetworkHistory("90d");
    } catch (e) {
      console.error("Failed to load 90d history for stats views:", e);
    }
  });

  // Derived lists and pagination details
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

  let ruleTrackedConsumers = $derived(() => {
    let consumers = [];
    for (let rule of rules) {
      if (!rule.active) continue;
      let name = rule.target;
      let usageItem = (rule.ruleType === "app" ? statsUsage?.apps : statsUsage?.sites)?.find(u => 
        (rule.ruleType === "site" ? cleanDomain(u.name) === cleanDomain(name) : u.name.toLowerCase() === name.toLowerCase())
      );
      let totalUsageSeconds = usageItem ? usageItem.seconds : 0;
      
      let ruleStat = statsRuleStats?.find(rs => rs.ruleId === rule.id);
      let timesBlocked = ruleStat ? ruleStat.timesBlocked : 0;
      let timesBypassed = ruleStat ? ruleStat.timesBypassed : 0;
      
      let focusSeconds = Math.min(totalUsageSeconds, rule.limitSeconds);
      let distractionSeconds = Math.max(0, totalUsageSeconds - rule.limitSeconds);
      
      consumers.push({
        rule,
        name,
        totalUsageSeconds,
        focusSeconds,
        distractionSeconds,
        timesBlocked,
        timesBypassed,
        isDistracting: rule.enforcement === "hard",
        isNeutral: rule.enforcement === "warn",
        isProductive: rule.enforcement === "soft"
      });
    }
    return consumers.sort((a,b) => b.totalUsageSeconds - a.totalUsageSeconds);
  });

  // Formatting utility helpers
  function getRuleRemainingSeconds(rule: Rule) {
    const todayStr = new Date().toISOString().slice(0, 10);
    const limit = rule.extraLimitDate === todayStr
      ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
      : rule.limitSeconds;
    // Note: Since stats view doesn't have live elapsed tracking, we match against simple limits
    return Math.max(0, limit);
  }

  function getTargetRule(target: string, type: "apps" | "sites") {
    const mappedType = type === "apps" ? "app" : "site";
    return rules.find(
      (r: Rule) =>
        r.active &&
        r.ruleType === mappedType &&
        (mappedType === "site"
          ? cleanDomain(r.target) === cleanDomain(target)
          : r.target.toLowerCase() === target.toLowerCase()),
    );
  }

  function formatDuration(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m`;
    return `${safeSeconds}s`;
  }

  function formatBytes(bytes: number) {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${bytes} B`;
  }

  function formatDateLabel(date: string) {
    return new Intl.DateTimeFormat(undefined, { weekday: "short" }).format(
      new Date(`${date}T00:00:00`),
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

  function consumerTotal(consumer: { downloadBytes: number; uploadBytes: number }) {
    return consumer.downloadBytes + consumer.uploadBytes;
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

  function getNetworkHeatmapColor(bytes: number) {
    if (bytes > 1_073_741_824) return "bg-violet-400 shadow-[0_0_5px_rgba(167,139,250,0.5)]";
    if (bytes > 524_288_000) return "bg-violet-500";
    if (bytes > 104_857_600) return "bg-violet-700";
    if (bytes > 1_048_576) return "bg-violet-900";
    return "bg-slate-900";
  }
</script>

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
            statsRange = range.key as any;
          }}
        >
          {range.label}
        </button>
      {/each}
    </div>
  </header>

  <!-- Sub-navigation Tabs -->
  <div class="flex border-b border-slate-800/80">
    <button
      class="px-5 py-3 text-sm font-bold border-b-2 transition-all duration-200 flex items-center gap-2
        {statsSubTab === 'habits' ? 'border-amber-400 text-amber-300' : 'border-transparent text-slate-400 hover:text-slate-200'}"
      type="button"
      onclick={() => statsSubTab = "habits"}
    >
      <Flame size={16} /> Activity &amp; Habits
    </button>
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

        <div class="mt-5 space-y-3">
          {#if filteredStatsScreenList.length}
            {#each paginatedStatsScreenList as item}
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

                  <div class="flex items-center gap-3 shrink-0">
                    <span class="text-sm font-black text-slate-100 font-mono">
                      {formatDuration(item.seconds)}
                    </span>

                    {#if targetRule}
                      <span class="text-[10px] font-black px-2 py-0.5 rounded-full border bg-teal-500/10 text-teal-300 border-teal-500/20">
                        Limited
                      </span>
                    {:else}
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

                <div class="w-full bg-slate-950 h-1.5 rounded-full overflow-hidden">
                  <div 
                    class="h-full rounded-full bg-gradient-to-r {isApp ? 'from-purple-500 to-teal-400' : 'from-teal-500 to-emerald-400'}"
                    style={`width: ${Math.max(4, Math.min(100, (item.seconds / Math.max(1, statsUsage?.totalSeconds ?? 0)) * 100))}%`}
                  ></div>
                </div>
              </div>
            {/each}

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
            <IconBadge icon={Wifi} tone="purple" label="Network Consumers" />
            <div>
              <h2 class="text-lg font-bold">Attributed Traffic</h2>
              <p class="text-sm text-slate-500">Bandwidth consumers logged during sample intervals</p>
            </div>
          </div>

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

      <!-- Heatmap section -->
      <section class="silo-card p-6">
        <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between border-b border-slate-800/60 pb-4 mb-6">
          <div class="flex items-center gap-3">
            <IconBadge icon={Calendar} tone="teal" label="Activity Calendar" />
            <div>
              <h2 class="text-[17px] font-bold text-slate-100">Activity Heatmap</h2>
              <p class="text-xs text-slate-500 mt-0.5">Last 12 weeks</p>
            </div>
          </div>

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
          {#if timeline?.days.length}
            <div class="mt-8 flex flex-col justify-center items-start w-full">
              <div class="grid grid-flow-col grid-rows-7 gap-[5px]">
                {#each timeline.days.slice(-84) as day}
                  <span
                    class="h-[11px] w-[11px] rounded-full transition-all duration-200 hover:scale-125 hover:ring-2 hover:ring-teal-400/50
                      {day.totalSeconds > 14400
                        ? 'bg-teal-400 shadow-[0_0_5px_rgba(45,212,191,0.5)]'
                        : day.totalSeconds > 7200
                          ? 'bg-teal-500'
                          : day.totalSeconds > 3600
                            ? 'bg-teal-700'
                            : day.totalSeconds > 0
                              ? 'bg-teal-900'
                              : 'bg-slate-900/60'}"
                    title={`${day.date}: ${formatDuration(day.totalSeconds)} focus time`}
                  ></span>
                {/each}
              </div>
              <div class="mt-8 relative flex w-full items-center justify-center text-[11px] font-medium text-slate-500 px-1">
                <span class="absolute left-1">Less</span>
                <div class="flex gap-1.5">
                  <span class="h-[10px] w-[10px] bg-slate-900/60 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-teal-900 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-teal-700 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-teal-500 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-teal-400 rounded-full"></span>
                </div>
                <span class="absolute right-1">More</span>
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
          {#if heatmapNetworkHistory.length}
            <div class="mt-8 flex flex-col justify-center items-start w-full">
              <div class="grid grid-flow-col grid-rows-7 gap-[5px]">
                {#each heatmapNetworkHistory.slice(-84) as day}
                  {@const dayBytes = day.downloadBytes + day.uploadBytes}
                  <span
                    class="h-[11px] w-[11px] rounded-full transition-all duration-200 hover:scale-125 hover:ring-2 hover:ring-violet-400/50
                      {getNetworkHeatmapColor(dayBytes)}"
                    title={`${day.date}: ${formatBytes(dayBytes)} total data`}
                  ></span>
                {/each}
              </div>
              <div class="mt-8 relative flex w-full items-center justify-center text-[11px] font-medium text-slate-500 px-1">
                <span class="absolute left-1">Less</span>
                <div class="flex gap-1.5">
                  <span class="h-[10px] w-[10px] bg-slate-900/60 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-violet-900 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-violet-700 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-violet-500 rounded-full"></span>
                  <span class="h-[10px] w-[10px] bg-violet-400 rounded-full"></span>
                </div>
                <span class="absolute right-1">More</span>
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

      <!-- Focus Trend Chart -->
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
  {:else if statsSubTab === "habits"}
    {@const consumers = ruleTrackedConsumers()}
    {@const totalFocus = consumers.reduce((sum, c) => sum + c.focusSeconds, 0)}
    {@const totalDistraction = consumers.reduce((sum, c) => sum + c.distractionSeconds, 0)}
    {@const totalBlocks = consumers.reduce((sum, c) => sum + c.timesBlocked, 0)}
    {@const totalBypasses = consumers.reduce((sum, c) => sum + c.timesBypassed, 0)}
    <div class="space-y-6" transition:fade={{ duration: 100 }}>
      <!-- Habits Metrics -->
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          icon={Target}
          title="Total Focus Time"
          value={formatDuration(totalFocus)}
          trend="Time within limits"
          tone="teal"
        />
        <MetricCard
          icon={Flame}
          title="Distraction Time"
          value={formatDuration(totalDistraction)}
          trend="Time over limits"
          tone="yellow"
        />
        <MetricCard
          icon={Award}
          title="Total Blocks"
          value={String(totalBlocks)}
          caption="Times limits enforced"
          tone="purple"
        />
        <MetricCard
          icon={Clock}
          title="Limit Bypasses"
          value={String(totalBypasses)}
          caption="Times time was extended"
          tone="neutral"
        />
      </div>

      <section class="silo-card p-6">
        <div class="flex items-center justify-between border-b border-slate-800/60 pb-4 mb-5">
          <div>
            <h2 class="text-lg font-bold">Per-app Focus &amp; Distraction</h2>
            <p class="text-sm text-slate-500">Activity breakdown for tracked rules</p>
          </div>
        </div>

        <div class="space-y-4">
          {#each consumers as consumer}
            {@const focusPct = consumer.totalUsageSeconds > 0 ? (consumer.focusSeconds / consumer.totalUsageSeconds) * 100 : 0}
            {@const distractionPct = consumer.totalUsageSeconds > 0 ? (consumer.distractionSeconds / consumer.totalUsageSeconds) * 100 : 0}
            <div class="p-4 rounded-xl border border-slate-800 bg-slate-900/50 flex flex-col gap-3 transition-colors hover:bg-slate-800/50 hover:border-slate-700">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="flex h-10 w-10 items-center justify-center rounded-lg {consumer.isDistracting ? 'bg-amber-500/10 text-amber-400' : consumer.isNeutral ? 'bg-violet-500/10 text-violet-400' : 'bg-teal-500/10 text-teal-400'}">
                    {#if consumer.rule.ruleType === "site"}
                      <Globe size={20} />
                    {:else}
                      <Monitor size={20} />
                    {/if}
                  </div>
                  <div>
                    <h3 class="font-bold text-slate-200">{consumer.name}</h3>
                    <p class="text-xs font-semibold mt-0.5 {consumer.isDistracting ? 'text-amber-400' : consumer.isNeutral ? 'text-violet-400' : 'text-teal-400'}">
                      {consumer.isProductive ? 'Productive' : consumer.isNeutral ? 'Neutral' : 'Distracting'} Rule
                    </p>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-sm font-bold text-slate-300">
                    Limit {Math.round(consumer.rule.limitSeconds / 60)}m
                  </p>
                  <p class="text-[11px] text-slate-500 mt-1 flex gap-1.5 justify-end font-semibold">
                    {#if consumer.timesBlocked > 0}
                      <span class="text-rose-400 bg-rose-500/10 px-1.5 py-0.5 rounded shadow-sm border border-rose-500/20">Blocked {consumer.timesBlocked}x</span>
                    {/if}
                    {#if consumer.timesBypassed > 0}
                      <span class="text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded shadow-sm border border-amber-500/20">Bypassed {consumer.timesBypassed}x</span>
                    {/if}
                  </p>
                </div>
              </div>

              <!-- Dual-color Ratio Bar -->
              <div class="flex flex-col gap-1.5 mt-1">
                <div class="flex items-center justify-between text-[11px] font-bold px-1">
                  <span class="text-teal-400">Focus: {formatDuration(consumer.focusSeconds)}</span>
                  {#if consumer.distractionSeconds > 0}
                    <span class="text-amber-400">Distraction: {formatDuration(consumer.distractionSeconds)}</span>
                  {/if}
                </div>
                <div class="flex h-2 w-full overflow-hidden rounded-full bg-slate-800 shadow-inner">
                  {#if consumer.totalUsageSeconds > 0}
                    <div class="bg-teal-400 transition-all duration-500 border-r border-slate-900" style="width: {focusPct}%"></div>
                    <div class="bg-amber-400 transition-all duration-500" style="width: {distractionPct}%"></div>
                  {:else}
                    <div class="h-full w-full bg-slate-800/50"></div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
          {#if consumers.length === 0}
             <EmptyState
                icon={Target}
                title="No tracked habits"
                message="Add rules for apps and sites to start tracking your focus and distraction habits."
              />
          {/if}
        </div>
      </section>
    </div>
  {/if}
</section>
