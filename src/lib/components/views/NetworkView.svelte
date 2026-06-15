<script lang="ts">
  import { fade } from "svelte/transition";
  import {
    Wifi,
    FileDown,
    Download,
    Upload,
    HardDrive,
    Monitor,
    Globe,
  } from "lucide-svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import TrendChart from "$lib/components/TrendChart.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import type { BootStatus, AppSnapshot, Settings, DataUsageReport, UsageDayBytes, DataConsumer } from "$lib/siloApi";

  let {
    snapshot,
    settings,
    dataUsage,
    exportUsage,
    exporting,
    dataRange = $bindable(),
    loadDataUsage,
    liveNetworkSamples,
    networkHistoryRange = $bindable(),
    changeNetworkHistoryRange,
    networkHistory,
    openMoreScreen
  } = $props<{
    snapshot: AppSnapshot | null;
    settings: Settings | null;
    dataUsage: DataUsageReport | null;
    exportUsage: () => Promise<void>;
    exporting: boolean;
    dataRange: "7d" | "30d" | "90d";
    loadDataUsage: (range: "7d" | "30d" | "90d") => Promise<void>;
    liveNetworkSamples: Array<{ time: string; down: number; up: number }>;
    networkHistoryRange: "7d" | "30d";
    changeNetworkHistoryRange: (range: "7d" | "30d") => Promise<void>;
    networkHistory: UsageDayBytes[];
    openMoreScreen: (title: string, rows: DataConsumer[]) => Promise<void>;
  }>();

  // Helper calculations
  function totalDataBytes() {
    return (
      (dataUsage?.totalDownloadBytes ?? 0) + (dataUsage?.totalUploadBytes ?? 0)
    );
  }

  function consumerTotal(consumer: DataConsumer) {
    return consumer.downloadBytes + consumer.uploadBytes;
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
    return new Intl.DateTimeFormat(undefined, { weekday: "short" }).format(
      new Date(`${date}T00:00:00`),
    );
  }

  function historyChartLabels() {
    return networkHistory.map((day: UsageDayBytes) => formatDateLabel(day.date));
  }

  function liveChartLabels() {
    const totalSamples = liveNetworkSamples.length;
    const interval = settings?.sampleIntervalSeconds ?? 5;

    return liveNetworkSamples.map((_, i) => {
      const secondsAgo = (totalSamples - 1 - i) * interval;
      if (secondsAgo === 0) return "Now";
      if (secondsAgo < 60) return `-${secondsAgo}s`;
      
      const m = Math.floor(secondsAgo / 60);
      const s = secondsAgo % 60;
      return s === 0 ? `-${m}m` : `-${m}m ${s}s`;
    });
  }

  function liveChartData() {
    return [
      {
        label: "Download (MB/s)",
        data: liveNetworkSamples.map((s: { time: string; down: number; up: number }) => s.down),
        backgroundColor: "rgba(20, 184, 166, 0.15)",
        borderColor: "#2dd4bf",
        borderWidth: 2,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
      },
      {
        label: "Upload (MB/s)",
        data: liveNetworkSamples.map((s: { time: string; down: number; up: number }) => s.up),
        backgroundColor: "rgba(139, 92, 246, 0.15)",
        borderColor: "#a78bfa",
        borderWidth: 2,
        fill: true,
        tension: 0.3,
        pointRadius: 0,
      },
    ];
  }

  function historyChartData() {
    return [
      {
        label: "Total Usage (MB)",
        data: networkHistory.map((day: UsageDayBytes) =>
          Math.round((day.downloadBytes + day.uploadBytes) / 1_048_576),
        ),
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
</script>

<section class="mx-auto max-w-6xl space-y-6">
  <header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
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
      <span class="flex items-center gap-2">
        <span class="h-2 w-2 rounded-full bg-emerald-400"></span> Live network speed
      </span>
      <span>Sample interval: {settings?.sampleIntervalSeconds ?? 5}s</span>
    </div>
    <div class="mt-8 grid gap-8 md:grid-cols-2">
      <div>
        <p class="flex items-center gap-2 text-sm text-slate-500">
          <Download size={16} /> Download
        </p>
        <p class="mt-2 text-5xl font-black text-teal-350">
          {formatBps(snapshot?.networkSpeed?.downloadBps ?? 0)}
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
          {formatBps(snapshot?.networkSpeed?.uploadBps ?? 0)}
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
          maxXTicks={7}
        />
      </div>
    {:else}
      <div class="mt-8 h-1 rounded-full bg-teal-400/70"></div>
    {/if}
  </section>

  <section class="silo-card p-6">
    <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h2 class="text-lg font-bold">Usage breakdown</h2>
        <p class="text-sm text-slate-500">
          See how much data each app and site used over the selected period
        </p>
      </div>
      <div class="flex rounded-lg bg-slate-800 p-1">
        {#each [
          { key: "today", label: "Day" },
          { key: "7d", label: "Week" },
          { key: "30d", label: "Month" },
          { key: "90d", label: "Quarter" }
        ] as range}
          <button
            class="rounded-md px-3 py-2 text-xs font-bold transition {dataRange === range.key
              ? 'bg-slate-950 text-slate-100'
              : 'text-slate-400 hover:text-slate-100'}"
            type="button"
            onclick={() => loadDataUsage(range.key as any)}
          >
            {range.label}
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
        <p class="mt-1 text-2xl font-black">
          {dataRange === "today" ? "Day" : dataRange === "7d" ? "Week" : dataRange === "30d" ? "Month" : "Quarter"}
        </p>
      </div>
    </div>
    {#if totalDataBytes() > 0}
      {@const down = dataUsage?.totalDownloadBytes ?? 0}
      {@const up = dataUsage?.totalUploadBytes ?? 0}
      {@const total = down + up}
      {@const downPct = total > 0 ? (down / total) * 100 : 0}
      {@const upPct = total > 0 ? (up / total) * 100 : 0}
      
      <div class="mt-8 mb-2">
        <div class="flex justify-between text-[11px] font-bold text-slate-400 uppercase tracking-wider mb-3 px-1">
          <div class="flex items-center gap-2">
            <span class="w-2.5 h-2.5 rounded-sm bg-teal-400 shadow-[0_0_6px_rgba(45,212,191,0.4)]"></span> Download ({downPct.toFixed(1)}%)
          </div>
          <div class="flex items-center gap-2">
            Upload ({upPct.toFixed(1)}%) <span class="w-2.5 h-2.5 rounded-sm bg-violet-500 shadow-[0_0_6px_rgba(139,92,246,0.4)]"></span>
          </div>
        </div>
        <div class="w-full h-3.5 rounded-full bg-slate-900 overflow-hidden flex shadow-inner">
          <div class="h-full bg-teal-400 transition-all duration-500" style="width: {downPct}%"></div>
          <div class="h-full bg-violet-500 transition-all duration-500" style="width: {upPct}%"></div>
        </div>
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
    <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h2 class="text-lg font-bold">Usage History</h2>
        <p class="text-sm text-slate-500">
          Total data usage (upload + download) per day
        </p>
      </div>
      <div class="flex rounded-lg bg-slate-800 p-1">
        {#each ["7d", "30d"] as range}
          <button
            class="rounded-md px-3 py-2 text-xs font-bold transition {networkHistoryRange === range
              ? 'bg-slate-950 text-slate-100'
              : 'text-slate-400 hover:text-slate-100'}"
            type="button"
            onclick={() => changeNetworkHistoryRange(range as any)}
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

{#snippet ConsumerList(
  title: string,
  rows: DataConsumer[],
  label: string,
  icon: any,
)}
  <section class="silo-card p-7 flex flex-col justify-between">
    <div>
      <div class="flex items-center justify-between gap-3 mb-8">
        <div class="flex items-center gap-2.5">
          <svelte:component this={icon} size={18} class="text-teal-400" />
          <h2 class="text-[15px] font-bold text-slate-100">{title}</h2>
        </div>
        <span class="text-[11px] text-slate-500 font-semibold">{dataRange}</span>
      </div>
      
      <div class="space-y-6">
        {#if rows.length}
          {@const maxTotal = Math.max(1, ...rows.map(r => consumerTotal(r)))}
          {#each rows.slice(0, 5) as row}
            {@const total = consumerTotal(row)}
            {@const widthPct = Math.max(0.5, (total / maxTotal) * 100)}
            {@const downSegmentPct = total > 0 ? (row.downloadBytes / total) * 100 : 0}
            {@const upSegmentPct = total > 0 ? (row.uploadBytes / total) * 100 : 0}
            
            <div class="group">
              <div class="flex items-start justify-between gap-3 mb-2">
                <div class="flex items-center gap-3.5 min-w-0">
                  <!-- Icon placeholder / Fallback -->
                  <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-slate-800/40 text-slate-300 shrink-0 border border-slate-700/30">
                     <svelte:component this={icon} size={15} />
                  </div>
                  <div class="min-w-0 mt-0.5">
                    <p class="truncate text-[14px] font-bold text-slate-100 leading-tight">{row.name}</p>
                    <p class="text-[10px] text-slate-500 font-medium tracking-wide mt-1">
                      {label === "Apps" ? "Application" : "Network Domain"}
                    </p>
                  </div>
                </div>
                
                <div class="text-right shrink-0 mt-0.5">
                  <p class="text-[14px] font-black text-slate-100 leading-tight">
                    {formatBytes(total)}
                  </p>
                  <p class="text-[10px] text-slate-500 font-medium tracking-wide mt-1">
                    ↓ {formatBytes(row.downloadBytes)} · ↑ {formatBytes(row.uploadBytes)}
                  </p>
                </div>
              </div>
              
              <!-- Segmented Progress Bar -->
              <div class="w-full h-[5px] rounded-full bg-slate-900 overflow-hidden relative mt-3">
                <div class="h-full flex rounded-full overflow-hidden absolute left-0 top-0 transition-all duration-500" style="width: {widthPct}%;">
                  <div class="h-full bg-teal-400" style="width: {downSegmentPct}%"></div>
                  <div class="h-full bg-violet-500" style="width: {upSegmentPct}%"></div>
                </div>
              </div>
              
              <div class="text-right mt-1.5">
                <span class="text-[9px] font-semibold text-slate-500 tracking-wide">{widthPct.toFixed(1)}% of top</span>
              </div>
            </div>
          {/each}
        {:else}
          <EmptyState
            compact
            icon={HardDrive}
            title="No usage records"
            message="Data usage will appear after attribution is implemented."
          />
        {/if}
      </div>
    </div>
    
    {#if rows.length > 5}
      <div class="mt-8 border-t border-slate-800/60 pt-5 text-center">
        <button
          class="text-[11px] font-bold text-slate-400 hover:text-teal-400 transition-colors uppercase tracking-widest"
          type="button"
          onclick={() => openMoreScreen(title, rows)}
        >
          View all {label}
        </button>
      </div>
    {/if}
  </section>
{/snippet}
