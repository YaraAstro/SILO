<script lang="ts">
  import {
    Power,
    Calendar,
    Moon,
    Database,
    Download,
    FileDown,
    RotateCcw,
    Keyboard,
    Info,
    RefreshCw,
  } from "lucide-svelte";
  import IconBadge from "$lib/components/IconBadge.svelte";
  import ToggleSwitch from "$lib/components/ToggleSwitch.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { check } from "@tauri-apps/plugin-updater";
  import { ask, message } from "@tauri-apps/plugin-dialog";
  import { siloApi } from "$lib/siloApi";
  import type { Settings, BootStatus } from "$lib/siloApi";

  let {
    settings = $bindable(),
    boot = $bindable(),
    showToast,
  } = $props<{
    settings: Settings | null;
    boot: BootStatus | null;
    showToast: (message: string, type?: "success" | "error" | "info") => void;
  }>();

  let savingSettings = $state(false);
  let exporting = $state(false);
  let exportPath = $state("");
  let checkingUpdate = $state(false);

  // Handler functions
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

  import { open } from "@tauri-apps/plugin-dialog";

  async function exportUsage() {
    exporting = true;
    try {
      const selectedDirPath = await open({
        directory: true,
        multiple: false,
        title: "Select Export Directory"
      });
      if (selectedDirPath) {
        const result = await siloApi.exportUsageData("30d", selectedDirPath as string);
        exportPath = result.filePath;
        showToast("Usage data exported successfully!", "success");
      }
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      exporting = false;
    }
  }

  async function exportLogs() {
    exporting = true;
    try {
      const selectedDirPath = await open({
        directory: true,
        multiple: false,
        title: "Select Export Directory"
      });
      if (selectedDirPath) {
        const result = await siloApi.exportLogs("30d", selectedDirPath as string);
        exportPath = result.filePath;
        showToast("Logs exported successfully!", "success");
      }
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      exporting = false;
    }
  }

  async function handleOpenUrl(url: string) {
    try {
      await openUrl(url);
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    }
  }

  async function checkForUpdates() {
    checkingUpdate = true;
    try {
      const update = await check();
      if (update) {
        const yes = await ask(
          `Update to ${update.version} is available!\n\nRelease notes:\n${update.body || 'No release notes provided.'}\n\nDo you want to install it now?`,
          {
            title: "Update Available",
            kind: "info",
          }
        );
        if (yes) {
          showToast("Downloading update...", "info");
          await update.downloadAndInstall();
          showToast("Update installed successfully. Please restart SILO.", "success");
        }
      } else {
        await message("You are on the latest version.", {
          title: "No Update Available",
          kind: "info",
        });
      }
    } catch (error) {
      showToast(toErrorMessage(error), "error");
    } finally {
      checkingUpdate = false;
    }
  }

  function toErrorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }
</script>

{#if settings}
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
        <p class="mt-3 text-sm text-slate-505">
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
        <p class="mt-4 break-all rounded-lg bg-slate-950/60 p-3 text-sm text-slate-400">
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
          { keys: ["Alt + D"], desc: "Go to Dashboard" },
          { keys: ["Alt + R"], desc: "Go to Rules & Limits" },
          { keys: ["Alt + S"], desc: "Go to Statistics" },
          { keys: ["Alt + N"], desc: "Go to Network Usage" },
          { keys: ["Alt + P", "Alt + ,"], desc: "Go to Settings" },
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

    <section class="silo-card p-6 relative overflow-hidden">
      <!-- Subtle gradient background glow -->
      <div class="absolute -right-16 -top-16 w-36 h-36 bg-gradient-to-br from-teal-500/10 to-purple-500/10 rounded-full blur-2xl pointer-events-none"></div>
      
      <div class="flex items-center gap-3">
        <IconBadge icon={Info} tone="teal" label="About" />
        <h2 class="text-lg font-bold">About SILO</h2>
      </div>
      
      <div class="mt-6 flex flex-col md:flex-row gap-6 items-start md:items-center justify-between">
        <div class="space-y-3">
          <div class="flex items-center gap-2">
            <span class="text-2xl font-black bg-gradient-to-r from-teal-400 to-purple-400 bg-clip-text text-transparent">SILO</span>
            <span class="text-xs font-mono font-bold bg-slate-800 text-teal-400 px-2 py-0.5 rounded border border-teal-500/10">v0.5.0-pre</span>
            <button
              class="ml-2 flex items-center gap-1 rounded bg-slate-800 px-2 py-1 text-xs font-semibold text-slate-300 hover:bg-slate-700 hover:text-white transition-colors"
              onclick={checkForUpdates}
              disabled={checkingUpdate}
            >
              <RefreshCw size={12} class={checkingUpdate ? 'animate-spin' : ''} />
              {checkingUpdate ? 'Checking...' : 'Check for Updates'}
            </button>
          </div>
          <p class="text-sm text-slate-300 max-w-lg leading-relaxed">
            A local-first, privacy-focused productivity and digital wellness platform. Monitor screen time, control distractions, and analyze internet usage in real-time.
          </p>
          <div class="text-xs text-slate-400">
            Developed by <strong class="text-slate-200">YaraAstro</strong>
          </div>
        </div>
        
        <div class="grid grid-cols-2 gap-3 shrink-0 w-full md:w-auto">
          <div class="p-3 bg-slate-950/45 border border-slate-900 rounded-lg text-center">
            <div class="text-[10px] text-slate-500 uppercase font-bold tracking-wider">Database</div>
            <div class="text-xs text-slate-300 font-semibold mt-1">SQLite Local</div>
          </div>
          <div class="p-3 bg-slate-950/45 border border-slate-900 rounded-lg text-center">
            <div class="text-[10px] text-slate-500 uppercase font-bold tracking-wider">Engine</div>
            <div class="text-xs text-slate-300 font-semibold mt-1">Rust &amp; Svelte</div>
          </div>
        </div>
      </div>
      
      <div class="mt-6 pt-5 border-t border-slate-800/60 flex flex-wrap gap-4 items-center justify-between text-xs text-slate-500">
        <div class="flex items-center gap-2">
          <span>Built with Tauri &amp; Win32 ETW</span>
          <span class="text-slate-700">•</span>
          <span>Personal Use License</span>
        </div>
        <div class="flex items-center gap-3">
          <button 
            class="hover:text-teal-400 transition font-semibold cursor-pointer"
            onclick={() => handleOpenUrl("https://github.com/YaraAstro/SILO")}
          >
            GitHub Repository
          </button>
          <span class="text-slate-700">•</span>
          <button 
            class="hover:text-teal-400 transition font-semibold cursor-pointer"
            onclick={() => handleOpenUrl("https://github.com/YaraAstro/SILO/issues")}
          >
            Report Issue
          </button>
        </div>
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
