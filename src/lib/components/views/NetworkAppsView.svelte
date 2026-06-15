<script lang="ts">
  import { fade } from "svelte/transition";
  import { Monitor, Download, Upload, Search } from "lucide-svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import type { DataUsageReport, DataConsumer } from "$lib/siloApi";

  let {
    activeView = $bindable(),
    detailRange = $bindable(),
    loadDetailUsage,
    detailUsage,
    moreModalSearch = $bindable(),
  } = $props<{
    activeView: string;
    detailRange: "today" | "7d" | "30d";
    loadDetailUsage: (range: "today" | "7d" | "30d") => Promise<void>;
    detailUsage: DataUsageReport | null;
    moreModalSearch: string;
  }>();

  // Local helper calculations
  let filteredMoreRows = $derived(
    (detailUsage?.apps ?? []).filter((row: DataConsumer) =>
      row.name.toLowerCase().includes(moreModalSearch.toLowerCase()),
    ),
  );

  // Pagination state
  let currentPage = $state(1);
  const pageSize = 10;

  $effect(() => {
    moreModalSearch;
    detailRange;
    currentPage = 1;
  });

  let paginatedRows = $derived(
    filteredMoreRows.slice((currentPage - 1) * pageSize, currentPage * pageSize)
  );

  let totalPages = $derived(
    Math.max(1, Math.ceil(filteredMoreRows.length / pageSize))
  );

  function getDetailTotalBytes() {
    return (detailUsage?.apps ?? []).reduce(
      (sum: number, item: DataConsumer) => sum + item.downloadBytes + item.uploadBytes,
      0,
    );
  }

  function getDetailDownloadBytes() {
    return (detailUsage?.apps ?? []).reduce((sum: number, item: DataConsumer) => sum + item.downloadBytes, 0);
  }

  function getDetailUploadBytes() {
    return (detailUsage?.apps ?? []).reduce((sum: number, item: DataConsumer) => sum + item.uploadBytes, 0);
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
</script>

<section class="mx-auto max-w-5xl space-y-6" transition:fade={{ duration: 150 }}>
  <!-- Header Nav Row -->
  <header class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
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
    <div class="flex rounded-xl bg-slate-900/90 border border-slate-800 p-1 w-fit shadow-lg shadow-black/20 self-start md:self-auto">
      {#each [{ key: "today", label: "Today" }, { key: "7d", label: "Last 7 Days" }, { key: "30d", label: "Last 30 Days" }] as range}
        <button
          class="rounded-lg px-4 py-2 text-xs font-bold transition-all duration-200 {detailRange === range.key
            ? 'bg-teal-400 text-slate-950 font-black shadow-md'
            : 'text-slate-400 hover:text-slate-100'}"
          type="button"
          onclick={() => loadDetailUsage(range.key as any)}
        >
          {range.label}
        </button>
      {/each}
    </div>
  </header>

  <!-- Stats Overview Cards Grid -->
  <div class="grid gap-4 grid-cols-2 lg:grid-cols-4">
    <div class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80">
      <span class="text-xs font-bold text-slate-500 uppercase tracking-wider">Total Usage</span>
      <span class="text-2xl font-black text-slate-200 mt-2 block">{formatBytes(getDetailTotalBytes())}</span>
      <span class="text-xs text-slate-505 mt-1">Upload + Download</span>
    </div>
    <div class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80">
      <span class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1">
        <Download class="text-teal-400" size={13} />
        Download
      </span>
      <span class="text-2xl font-black text-teal-350 mt-2 block">{formatBytes(getDetailDownloadBytes())}</span>
      <span class="text-xs text-slate-505 mt-1">Received data</span>
    </div>
    <div class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80">
      <span class="text-xs font-bold text-slate-550 uppercase tracking-wider flex items-center gap-1">
        <Upload class="text-violet-400" size={13} />
        Upload
      </span>
      <span class="text-2xl font-black text-violet-300 mt-2 block">{formatBytes(getDetailUploadBytes())}</span>
      <span class="text-xs text-slate-505 mt-1">Sent data</span>
    </div>
    <div class="silo-card p-5 flex flex-col justify-between bg-slate-900/40 border border-slate-800/80">
      <span class="text-xs font-bold text-slate-550 uppercase tracking-wider">Active Apps</span>
      <span class="text-2xl font-black text-slate-200 mt-2 block">{filteredMoreRows.length}</span>
      <span class="text-xs text-slate-505 mt-1">Attributed this period</span>
    </div>
  </div>

  <!-- Main Content List Card -->
  <section class="silo-card p-6 space-y-6">
    <!-- Search Bar -->
    <div class="relative w-full">
      <Search class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-550" size={17} />
      <input
        class="silo-input pl-10.5 w-full bg-slate-950/40 focus:border-teal-400 transition-colors"
        placeholder="Search apps..."
        bind:value={moreModalSearch}
      />
    </div>

    <!-- Table Headers (Desktop) -->
    <div class="hidden sm:grid grid-cols-[1.5fr_1fr_1fr_1fr] px-4 py-2 border-b border-slate-800/60 text-xs font-extrabold text-slate-555 tracking-wider">
      <span>APPLICATION</span>
      <span class="text-right flex items-center justify-end gap-1"><Download size={11} /> DOWNLOAD</span>
      <span class="text-right flex items-center justify-end gap-1"><Upload size={11} /> UPLOAD</span>
      <span class="text-right">TOTAL</span>
    </div>

    <div class="space-y-3.5">
      {#if filteredMoreRows.length}
        {#each paginatedRows as row, i}
          {@const totalBytes = consumerTotal(row)}
          {@const sharePercentage = ((totalBytes / Math.max(1, getDetailTotalBytes())) * 100).toFixed(1)}
          {@const rankIndex = (currentPage - 1) * pageSize + i + 1}
          <div class="silo-card p-4 hover:border-slate-700/85 hover:bg-slate-900/30 transition-all duration-200 group flex flex-col gap-3">
            <div class="flex flex-col sm:grid sm:grid-cols-[1.5fr_1fr_1fr_1fr] items-start sm:items-center justify-between gap-2.5">
              <div class="flex items-center gap-3 min-w-0 w-full">
                <!-- Rank badge -->
                <span class="text-xs font-black px-2 py-1 bg-slate-800 text-slate-400 group-hover:bg-teal-400 group-hover:text-slate-950 rounded-md transition-colors min-w-[28px] text-center">
                  #{rankIndex}
                </span>
                <p class="truncate font-black text-slate-200 group-hover:text-teal-300 transition-colors">
                  {row.name}
                </p>
              </div>

              <!-- Down bytes -->
              <div class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right">
                <span class="sm:hidden font-semibold text-slate-500">Download</span>
                <span class="font-semibold text-teal-350/90">{formatBytes(row.downloadBytes)}</span>
              </div>

              <!-- Up bytes -->
              <div class="flex justify-between w-full sm:w-auto sm:justify-end text-sm text-slate-400 sm:text-right">
                <span class="sm:hidden font-semibold text-slate-500">Upload</span>
                <span class="font-semibold text-violet-350">{formatBytes(row.uploadBytes)}</span>
              </div>

              <!-- Total bytes & Share % -->
              <div class="flex justify-between w-full sm:w-auto sm:justify-end items-center gap-2 sm:text-right font-black text-slate-100">
                <span class="sm:hidden font-bold text-slate-500">Total</span>
                <div class="flex items-baseline gap-1.5 justify-end">
                  <span class="text-slate-100">{formatBytes(totalBytes)}</span>
                  <span class="text-xs text-slate-505 font-semibold">({sharePercentage}%)</span>
                </div>
              </div>
            </div>

            <!-- Distribution Progress Bar -->
            <div class="h-2 rounded-full bg-slate-950/70 overflow-hidden">
              <div
                class="h-full rounded-full bg-gradient-to-r from-teal-400 to-violet-500 transition-all duration-500"
                style={`width: ${Math.max(4, Math.min(100, (totalBytes / Math.max(1, getDetailTotalBytes())) * 100))}%`}
              ></div>
            </div>
          </div>
        {/each}
        
        {#if totalPages > 1}
          <div class="flex items-center justify-between border-t border-slate-800/60 pt-4 mt-2">
            <button
              class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
              type="button"
              disabled={currentPage === 1}
              onclick={() => currentPage -= 1}
            >
              Previous
            </button>
            <span class="text-xs text-slate-400 font-semibold">
              Page {currentPage} of {totalPages}
            </span>
            <button
              class="silo-button-secondary bg-slate-800 hover:bg-slate-700 px-3 py-1.5 text-xs rounded-lg transition disabled:opacity-40 disabled:hover:bg-slate-800"
              type="button"
              disabled={currentPage === totalPages}
              onclick={() => currentPage += 1}
            >
              Next
            </button>
          </div>
        {/if}
      {:else}
        <EmptyState compact icon={Search} title="No results found" message="Try a different search query." />
      {/if}
    </div>
  </section>
</section>
