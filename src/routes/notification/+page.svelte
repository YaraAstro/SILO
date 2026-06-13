<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { listen } from "@tauri-apps/api/event";
  import { Clock, CircleAlert, X, Globe, Monitor } from "lucide-svelte";

  let ruleId = $state<number | null>(null);
  let target = $state("");
  let enforcement = $state("");
  let limitSeconds = $state(0);
  let remainingSeconds = $state(0);
  let ruleType = $state("app");

  let isClosing = $state(false);

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    ruleId = params.get("ruleId") ? Number(params.get("ruleId")) : null;
    target = params.get("target") || "";
    enforcement = params.get("enforcement") || "";
    limitSeconds = params.get("limitSeconds") ? Number(params.get("limitSeconds")) : 0;
    remainingSeconds = params.get("remainingSeconds") ? Number(params.get("remainingSeconds")) : 0;
    
    // Guess rule type based on target naming (websites have dots and no .exe)
    if (target.includes(".") && !target.toLowerCase().endsWith(".exe")) {
      ruleType = "site";
    }

    // Listen to live updates from the backend to prevent window recreation lag
    const unlistenPromise = listen<number>("update_countdown", (event) => {
      remainingSeconds = event.payload;
    });

    // Auto-dismiss if remaining <= 0 (block has hit) and it's not interactive
    // Soft warning has no actions, so we auto-close after 8 seconds
    if (enforcement === "soft") {
      setTimeout(closeNotification, 8000);
    }

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });

  async function closeNotification() {
    isClosing = true;
    setTimeout(async () => {
      try {
        const appWindow = getCurrentWebviewWindow();
        await appWindow.close();
      } catch (err) {
        console.error("Failed to close window:", err);
      }
    }, 200);
  }

  async function handleAddTime(minutes: number) {
    if (ruleId === null) return;
    try {
      await invoke("add_rule_time", { id: ruleId, seconds: minutes * 60 });
      await closeNotification();
    } catch (err) {
      console.error("Failed to add time:", err);
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
</script>

<main class="h-screen w-screen bg-transparent p-3 select-none overflow-hidden font-sans">
  <div 
    class="h-full w-full rounded-2xl border border-slate-700 bg-slate-900/95 shadow-2xl p-4 flex flex-col justify-between transition-all duration-300 relative
           {isClosing ? 'opacity-0 scale-95' : 'opacity-100 scale-100'}"
  >
    <!-- Top-Right Close Button -->
    <button 
      type="button" 
      onclick={closeNotification}
      class="absolute top-3 right-3 text-slate-500 hover:text-slate-200 transition p-1 hover:bg-slate-800 rounded-lg"
      aria-label="Close notification"
    >
      <X size={16} />
    </button>

    <!-- Header Section -->
    <div class="flex items-start gap-3">
      <div 
        class="h-10 w-10 shrink-0 flex items-center justify-center rounded-xl 
               {enforcement === 'hard' ? 'bg-red-500/10 text-red-400' : enforcement === 'warn' ? 'bg-amber-500/10 text-amber-400' : 'bg-blue-500/10 text-blue-400'}"
      >
        {#if ruleType === 'site'}
          <Globe size={20} />
        {:else}
          <Monitor size={20} />
        {/if}
      </div>

      <div class="min-w-0 flex-1 pr-6">
        <h2 class="text-sm font-bold text-slate-100 truncate">
          {#if enforcement === 'hard'}
            {#if remainingSeconds <= 0}
              Focus Block: Closed
            {:else}
              Hard Block in {remainingSeconds}s
            {/if}
          {:else}
            {#if remainingSeconds <= 0}
              Limit Reached
            {:else}
              Remaining: {formatDuration(remainingSeconds)}
            {/if}
          {/if}
        </h2>
        <p class="text-xs text-slate-400 mt-0.5 truncate font-mono">
          {target}
        </p>
      </div>
    </div>

    <!-- Body / Content -->
    <div class="flex-1 my-2 flex items-center">
      <p class="text-xs text-slate-300 leading-normal">
        {#if enforcement === 'hard'}
          {#if remainingSeconds <= 0}
            Time limit exceeded. The application has been closed.
          {:else}
            Save your work immediately! Notepad / browser window is closing.
          {/if}
        {:else if enforcement === 'warn'}
          {#if remainingSeconds <= 0}
            Limit reached! Closed target application. Extend to keep using.
          {:else}
            Warning: Focus limit is approaching. Extend time to keep active.
          {/if}
        {:else if enforcement === 'soft'}
          Time limit reached today. Window minimized to help you stay focused.
        {/if}
      </p>
    </div>

    <!-- Actions Section -->
    {#if enforcement === 'warn' || (enforcement === 'hard' && remainingSeconds > 0)}
      <!-- Warn & countdown allows adding time if warn enforcement, wait! Hard block does NOT allow adding time. -->
      {#if enforcement === 'warn'}
        <div class="flex justify-end gap-1.5 pt-1 border-t border-slate-800">
          <button 
            type="button" 
            onclick={() => handleAddTime(15)}
            class="bg-teal-600/20 hover:bg-teal-600/30 text-teal-300 border border-teal-500/20 text-[10px] font-bold px-2.5 py-1.5 rounded-lg transition"
          >
            +15m
          </button>
          <button 
            type="button" 
            onclick={() => handleAddTime(30)}
            class="bg-teal-600/20 hover:bg-teal-600/30 text-teal-300 border border-teal-500/20 text-[10px] font-bold px-2.5 py-1.5 rounded-lg transition"
          >
            +30m
          </button>
          <button 
            type="button" 
            onclick={() => handleAddTime(60)}
            class="bg-teal-600/20 hover:bg-teal-600/30 text-teal-300 border border-teal-500/20 text-[10px] font-bold px-2.5 py-1.5 rounded-lg transition"
          >
            +1h
          </button>
        </div>
      {:else}
        <!-- Hard block countdown only has acknowledge/save-work warning close action -->
        <div class="flex justify-end pt-1 border-t border-slate-800">
          <button 
            type="button" 
            onclick={closeNotification}
            class="bg-red-600 hover:bg-red-700 text-white text-[10px] font-bold px-4 py-1.5 rounded-lg transition"
          >
            Save Work
          </button>
        </div>
      {/if}
    {:else}
      <!-- Soft warning and other static notifications only have dismiss -->
      <div class="flex justify-end pt-1 border-t border-slate-800">
        <button 
          type="button" 
          onclick={closeNotification}
          class="bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700/55 text-[10px] font-bold px-4 py-1.5 rounded-lg transition"
        >
          Acknowledge
        </button>
      </div>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    background-color: transparent !important;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
