<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import {
    House,
    Power,
    Globe,
    Monitor,
    Clock,
    Timer,
    Target,
    Activity,
    Wifi,
    Download,
    Upload,
    Sparkles,
    ChartColumn,
    RotateCcw,
  } from "lucide-svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import MetricCard from "$lib/components/MetricCard.svelte";
  import TrendChart from "$lib/components/TrendChart.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import { cleanDomain } from "$lib/utils/cleanDomain";
  import type { BootStatus, AppSnapshot, Rule, UsageReport, UsageTimeline, DataUsageReport, UsageDayBytes } from "$lib/siloApi";

  let {
    snapshot,
    boot,
    rules,
    usage = $bindable(),
    usageTab = $bindable(),
    isRefreshing = $bindable(),
    refreshKey = $bindable(),
    timeline,
    dataUsage,
    networkHistory,
    liveNetworkSamples,
    toggleFocus,
    handleRefresh,
    openMoreScreen
  } = $props<{
    snapshot: AppSnapshot | null;
    boot: BootStatus | null;
    rules: Rule[];
    usage: UsageReport | null;
    usageTab: "apps" | "sites";
    isRefreshing: boolean;
    refreshKey: number;
    timeline: UsageTimeline | null;
    dataUsage: DataUsageReport | null;
    networkHistory: UsageDayBytes[];
    liveNetworkSamples: Array<{ time: string; down: number; up: number }>;
    toggleFocus: () => Promise<void>;
    handleRefresh: () => Promise<void>;
    openMoreScreen: (title: string, rows: any[]) => Promise<void>;
  }>();

  // Dynamic calculations
  let focusScoreValue = $derived.by(() => {
    let score = 100;
    rules.forEach((rule: Rule) => {
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
    rules.forEach((rule: Rule) => {
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

  // Local helper functions
  function getRuleRemainingSeconds(rule: Rule) {
    const todayStr = new Date().toISOString().slice(0, 10);
    const limit = rule.extraLimitDate === todayStr
      ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
      : rule.limitSeconds;

    let elapsed = 0;
    if (rule.ruleType === "app") {
      const match = usage?.apps.find(
        (a: { name: string; seconds: number }) => a.name.toLowerCase() === rule.target.toLowerCase(),
      );
      if (match) elapsed = match.seconds;
      if (
        snapshot?.activeApp &&
        snapshot.activeApp.app.toLowerCase() === rule.target.toLowerCase()
      ) {
        elapsed += snapshot.activeApp.elapsedSeconds;
      }
    } else if (rule.ruleType === "site") {
      const match = usage?.sites?.find(
        (s: { name: string; seconds: number }) => cleanDomain(s.name) === cleanDomain(rule.target),
      );
      if (match) elapsed = match.seconds;
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
      (r: Rule) =>
        r.active &&
        r.ruleType === mappedType &&
        (mappedType === "site"
          ? cleanDomain(r.target) === cleanDomain(target)
          : r.target.toLowerCase() === target.toLowerCase()),
    );
  }

  function getRuleUsagePercentage(rule: Rule) {
    const remaining = getRuleRemainingSeconds(rule);
    const todayStr = new Date().toISOString().slice(0, 10);
    const limit = rule.extraLimitDate === todayStr
      ? rule.limitSeconds + (rule.extraLimitSeconds ?? 0)
      : rule.limitSeconds;
    if (limit <= 0) return 100;
    return Math.max(0, Math.min(100, ((limit - remaining) / limit) * 100));
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
    if (hours > 0)
      return `${hours}:${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
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
    return new Intl.DateTimeFormat(undefined, { weekday: "short" }).format(
      new Date(`${date}T00:00:00`),
    );
  }

  function weekDays() {
    return timeline?.days.slice(-7) ?? [];
  }

  function chartLabels() {
    return weekDays().map((day: { date: string; totalSeconds: number }) => formatDateLabel(day.date));
  }

  function focusChartData() {
    return [
      {
        label: "Tracked time",
        data: weekDays().map((day: { date: string; totalSeconds: number }) => Math.round(day.totalSeconds / 60)),
        backgroundColor: "rgba(20, 184, 166, 0.85)",
        borderColor: "#2dd4bf",
        borderRadius: 6,
      },
    ];
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

  function liveChartLabels() {
    return liveNetworkSamples.map((s: { time: string; down: number; up: number }) => s.time);
  }
</script>

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
            icon={snapshot?.activeApp?.site ? Globe : Monitor}
            tone={snapshot?.activeApp?.site ? "teal" : "purple"}
            label="Active application"
          />
          {#if snapshot?.activeApp?.app}
            <span class="flex items-center gap-1.5 text-xs text-teal-400 font-bold bg-teal-950/40 border border-teal-500/20 px-2 py-0.5 rounded-full">
              <span class="h-1.5 w-1.5 rounded-full bg-teal-400 animate-radar"></span>
              Live
            </span>
          {/if}
        </div>
        
        {#if snapshot?.activeApp?.site}
          <p class="mt-6 truncate text-2xl font-black text-slate-100 group-hover:text-teal-300 transition-colors" title={snapshot?.activeApp?.site}>
            {snapshot?.activeApp?.site}
          </p>
          <p class="mt-1 truncate text-xs text-slate-500 font-semibold uppercase tracking-wider">
            via {snapshot?.activeApp?.app}
          </p>
        {:else}
          <p class="mt-6 truncate text-2xl font-black text-slate-100 group-hover:text-purple-300 transition-colors" title={snapshot?.activeApp?.app || "No active app"}>
            {snapshot?.activeApp?.app || "No active app"}
          </p>
          <p class="mt-1 truncate text-xs text-slate-500 font-semibold uppercase tracking-wider">
            {snapshot?.activeApp?.title || "Current window"}
          </p>
        {/if}
      </div>
      <p class="mt-5 flex items-center gap-2 font-mono text-lg font-bold text-teal-300">
        <Clock size={18} />
        {formatClock(snapshot?.activeApp?.elapsedSeconds ?? 0)}
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
    <section class="silo-card p-7 flex flex-col justify-between relative overflow-hidden group hover:border-teal-500/30 transition-all duration-300 bg-[#1a1f26]">
      <div class="absolute -right-8 -top-8 h-20 w-20 rounded-full bg-teal-500/5 blur-xl group-hover:bg-teal-500/10 transition-colors"></div>
      
      <div>
        <div class="flex items-center justify-between gap-3 mb-8">
          <div class="flex items-center gap-2.5">
            <Wifi size={18} class="text-teal-400" />
            <h2 class="text-[15px] font-bold text-slate-100">Network</h2>
          </div>
          <span class="flex items-center gap-1.5 text-[11px] text-emerald-400/80 font-semibold tracking-wide">
            <span class="h-1.5 w-1.5 rounded-full bg-emerald-500 shadow-[0_0_5px_rgba(16,185,129,0.6)] animate-pulse"></span>
            Live
          </span>
        </div>
        
        <div class="grid grid-cols-2 gap-4 mb-10">
          <div>
            <div class="flex items-center gap-1.5 text-xs font-semibold text-teal-400/90 mb-1.5">
              <Download size={13} strokeWidth={3} /> Download
            </div>
            <div class="text-[28px] leading-none font-black text-slate-100 font-sans tracking-tight flex items-baseline gap-1.5">
              {formatBps(snapshot?.networkSpeed?.downloadBps ?? 0).split(' ')[0]} <span class="text-base font-bold">{formatBps(snapshot?.networkSpeed?.downloadBps ?? 0).split(' ')[1] || 'B/s'}</span>
            </div>
            <div class="text-[11px] font-medium text-slate-500/90 mt-2">
              Today {formatBytes(dataUsage?.totalDownloadBytes ?? 0)}
            </div>
          </div>
          <div>
            <div class="flex items-center gap-1.5 text-xs font-semibold text-purple-400/90 mb-1.5">
              <Upload size={13} strokeWidth={3} /> Upload
            </div>
            <div class="text-[28px] leading-none font-black text-slate-100 font-sans tracking-tight flex items-baseline gap-1.5">
              {formatBps(snapshot?.networkSpeed?.uploadBps ?? 0).split(' ')[0]} <span class="text-base font-bold">{formatBps(snapshot?.networkSpeed?.uploadBps ?? 0).split(' ')[1] || 'B/s'}</span>
            </div>
            <div class="text-[11px] font-medium text-slate-500/90 mt-2">
              Today {formatBytes(dataUsage?.totalUploadBytes ?? 0)}
            </div>
          </div>
        </div>
      </div>
      
      <div class="mt-auto">
        {#if true}
          {@const recentSamples = liveNetworkSamples.slice(-35)}
          {@const maxSpd = Math.max(0.01, ...recentSamples.map(r => r.down + r.up))}
          <div class="flex flex-col justify-end h-[72px] relative pb-0.5">
            <!-- Dashed baseline -->
            <div class="absolute bottom-[1px] left-0 right-0 border-b-[3px] border-dashed border-teal-500/30 z-0"></div>
            
            <div class="flex items-end justify-between gap-[5px] h-full z-10 relative">
              {#each Array.from({length: 35}) as _, i}
                {@const s = recentSamples[recentSamples.length - 35 + i]}
                {@const speed = s ? s.down + s.up : 0}
                {@const heightPct = speed > 0.001 ? Math.max(15, (speed / maxSpd) * 100) : 15}
                <div class="flex-1 bg-teal-400 rounded-t-[3px] rounded-b-[2px] transition-all duration-300 relative"
                     style="height: {heightPct}%; {speed > 0.001 ? 'box-shadow: 0 0 10px rgba(45,212,191,0.3); opacity: 1;' : 'opacity: 0.85;'}">
                </div>
              {/each}
            </div>
          </div>
          
          <div class="flex items-center justify-between text-[11px] font-medium text-slate-500/80 mt-4 px-0.5">
            <span class="flex items-center gap-1.5">
              <Activity size={12} strokeWidth={2.5} class="text-slate-500/70" /> 
              Last {Math.min(35, Math.max(1, recentSamples.length)) * 5}s
            </span>
            <span>Peak {maxSpd > 0.01 ? maxSpd.toFixed(1) : "0.0"} MB/s</span>
          </div>
        {/if}
      </div>
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
      
      <div class="mt-6 border-t border-slate-800 pt-4 text-right text-xl font-black text-teal-300 font-mono">
        {formatDuration(usage?.totalSeconds ?? 0)}
      </div>
    </section>
  </div>
</section>
