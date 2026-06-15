<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade, slide } from "svelte/transition";
  import {
    Lock,
    Search,
    Shield,
    Plus,
    RotateCcw,
    Sparkles,
    Clock,
    Timer,
    Info,
    Trash2,
    Monitor,
    Globe,
  } from "lucide-svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import { cleanDomain } from "$lib/utils/cleanDomain";
  import { siloApi, emptyRule } from "$lib/siloApi";
  import type { Rule, AppSnapshot, Settings, UsageReport } from "$lib/siloApi";

  let {
    rules = $bindable(),
    snapshot = $bindable(),
    settings,
    usage,
    activeView = $bindable(),
    showToast,
    isAuthorized = $bindable(false),
  } = $props<{
    rules: Rule[];
    snapshot: AppSnapshot | null;
    settings: Settings | null;
    usage: UsageReport | null;
    activeView: string;
    showToast: (message: string, type?: "success" | "error" | "info") => void;
    isAuthorized?: boolean;
  }>();

  // Rules-specific UI states
  let ruleSearch = $state("");
  let ruleDraft = $state<Rule>(emptyRule());
  let showRuleForm = $state(false);
  let showConfirmDeleteOverlay = $state(false);
  let ruleToDelete = $state<Rule | null>(null);
  let savingRule = $state(false);
  let showAppDropdown = $state(false);
  let showSiteDropdown = $state(false);
  let availableApps = $state<string[]>([]);

  // Lock screen specific states
  let pinInput = $state("");
  let lockTime = $state(new Date());

  // Clock interval for lock screen
  let clockTimer: any;
  onMount(() => {
    clockTimer = setInterval(() => {
      lockTime = new Date();
    }, 1000);
  });
  onDestroy(() => {
    if (clockTimer) clearInterval(clockTimer);
  });

  // Calculate greetings
  let greetingText = $derived.by(() => {
    const hour = new Date().getHours();
    if (hour < 12) return "Good Morning";
    if (hour < 17) return "Good Afternoon";
    if (hour < 22) return "Good Evening";
    return "Good Night";
  });

  // PIN validation
  function verifyPin() {
    const today = new Date();
    const dd = String(today.getDate()).padStart(2, "0");
    const mm = String(today.getMonth() + 1).padStart(2, "0");
    const yy = String(today.getFullYear()).slice(-2);
    const correctPin = `${dd}${mm}${yy}`;

    if (pinInput === correctPin) {
      isAuthorized = true;
      showToast("Access granted. Rules unlocked.", "success");
    } else {
      showToast("Incorrect PIN. Access denied.", "error");
      pinInput = "";
    }
  }

  function handleKeypadPress(num: string) {
    if (pinInput.length < 6) {
      pinInput += num;
      if (pinInput.length === 6) {
        verifyPin();
      }
    }
  }

  function handleBackspace() {
    pinInput = pinInput.slice(0, -1);
  }

  // Keyboard interceptor
  function handleKeyDown(e: KeyboardEvent) {
    if (!isAuthorized) {
      if (e.key >= "0" && e.key <= "9") {
        handleKeypadPress(e.key);
        e.preventDefault();
        return;
      }
      if (e.key === "Backspace") {
        handleBackspace();
        e.preventDefault();
        return;
      }
      if (e.key === "Escape") {
        activeView = "dashboard";
        pinInput = "";
        e.preventDefault();
        return;
      }
      // Block all other keys when locked
      e.preventDefault();
      return;
    }

    // Escape handles form clearing when rules dashboard is unlocked
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
    }
  }

  // API Call handlers
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
    const sitesList = (usage?.sites ?? []).map((s: { name: string; seconds: number }) => s.name);
    const query = ruleDraft.target.trim().toLowerCase();
    if (!query) return sitesList;
    return sitesList.filter((site: string) => site.toLowerCase().includes(query));
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
      rules = await siloApi.getRules();
      snapshot = await siloApi.getAppState();
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
      rules = await siloApi.getRules();
      snapshot = await siloApi.getAppState();
      showToast(
        `Rule for ${rule.target} ${updated.active ? "activated" : "paused"}.`,
        "success",
      );
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
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
      rules = await siloApi.getRules();
      snapshot = await siloApi.getAppState();
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
  }

  // Calculations
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

  function formatDuration(seconds: number) {
    const safeSeconds = Math.max(0, seconds);
    const hours = Math.floor(safeSeconds / 3600);
    const minutes = Math.floor((safeSeconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m`;
    return `${safeSeconds}s`;
  }

  function filteredRules() {
    const query = ruleSearch.trim().toLowerCase();
    if (!query) return rules;
    return rules.filter(
      (rule: Rule) =>
        rule.target.toLowerCase().includes(query) ||
        rule.ruleType.includes(query),
    );
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if !isAuthorized}
  <!-- Lock screen view (Keypad removed as requested) -->
  <div class="mx-auto max-w-xl py-6" transition:fade={{ duration: 150 }}>
    <section class="silo-card p-8 flex flex-col items-center justify-center text-center relative overflow-hidden bg-slate-900/60 shadow-2xl border-slate-800">
      <div class="absolute -right-16 -top-16 h-48 w-48 rounded-full bg-teal-500/5 blur-3xl animate-pulse-glow"></div>
      <div class="absolute -left-16 -bottom-16 h-48 w-48 rounded-full bg-purple-500/5 blur-3xl animate-pulse-glow"></div>

      <!-- Glowing Lock Badge -->
      <div class="relative mb-6 group">
        <div class="absolute -inset-4 bg-teal-500/10 rounded-full blur-xl animate-pulse"></div>
        <div class="absolute -inset-1 bg-gradient-to-r from-teal-500 to-purple-500 rounded-full opacity-60 blur-md group-hover:opacity-80 transition duration-500"></div>
        <div class="relative flex items-center justify-center w-16 h-16 rounded-full bg-slate-950 border border-slate-800 text-teal-400">
          <Lock size={24} />
        </div>
      </div>

      <!-- Clock & Date Display -->
      <div class="mb-6 space-y-1">
        <p class="text-[10px] font-black text-slate-500 uppercase tracking-widest">{greetingText}</p>
        <h2 class="text-3xl font-mono font-black text-slate-100 tracking-wider">
          {lockTime.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false })}
        </h2>
        <p class="text-[11px] text-teal-450 font-bold tracking-wide">
          {lockTime.toLocaleDateString("en-US", { weekday: "long", month: "short", day: "numeric" })}
        </p>
      </div>

      <!-- Header text -->
      <div class="mb-6 space-y-2">
        <h1 class="text-2xl font-black text-slate-100">Rules Protection</h1>
        <p class="text-xs text-slate-400 max-w-xs leading-relaxed">
          Rules configuration is locked. Enter your PIN to authorize modifications.
        </p>
      </div>

      <!-- PIN cells -->
      <div class="flex gap-2.5 mb-8 justify-center">
        {#each Array(6) as _, i}
          <div class="w-10 h-14 rounded-xl border flex items-center justify-center text-xl font-black font-mono transition-all duration-300 shadow-md relative overflow-hidden
            {pinInput.length > i 
              ? 'border-teal-500/60 bg-teal-500/10 text-teal-300 scale-105' 
              : pinInput.length === i 
                ? 'border-purple-500/60 bg-purple-500/5 text-purple-300 scale-[1.02] ring-1 ring-purple-500/30' 
                : 'border-slate-800 bg-slate-950/50 text-slate-600'}"
          >
            {#if pinInput.length === i}
              <div class="absolute inset-x-0 bottom-0 h-1 bg-purple-400 animate-pulse"></div>
            {/if}
            {#if pinInput.length > i}•{/if}
          </div>
        {/each}
      </div>

      <button 
        type="button" 
        class="text-xs font-bold text-slate-500 hover:text-teal-400 transition"
        onclick={() => { activeView = "dashboard"; }}
      >
        Cancel & Return to Dashboard
      </button>
    </section>
  </div>
{:else}
  <section class="mx-auto max-w-5xl space-y-6">
    <header class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
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
            <label class="inline-flex items-center gap-3 text-sm font-semibold text-slate-300">
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
{/if}

<!-- Delete Confirmation Overlay (inlined for self-contained view control) -->
{#if showConfirmDeleteOverlay && ruleToDelete}
  <div transition:fade={{ duration: 200 }} class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/85 backdrop-blur-md">
    <div class="silo-card max-w-md w-full p-8 border border-red-500/30 bg-slate-900/90 shadow-2xl text-center space-y-6">
      <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-red-500/10 text-red-400">
        <Trash2 size={32} />
      </div>
      <div class="space-y-2">
        <h2 class="text-2xl font-black text-slate-100">Delete Rule</h2>
        <p class="text-sm text-slate-400">
          Are you sure you want to delete the rule for this {ruleToDelete.ruleType}?
        </p>
      </div>
      <div class="rounded-lg bg-slate-950/50 p-4 border border-slate-800 font-mono">
        <span class="text-red-300 font-bold">{ruleToDelete.target}</span>
      </div>
      <div class="flex justify-center gap-3 pt-2">
        <button class="silo-button-secondary bg-slate-800 hover:bg-slate-700 text-slate-200 font-bold px-6 py-2.5 rounded-lg transition" type="button" onclick={cancelDeleteRule}>
          Cancel
        </button>
        <button class="silo-button bg-red-600 hover:bg-red-700 text-white font-bold px-6 py-2.5 rounded-lg transition" type="button" onclick={confirmDeleteRule}>
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}
