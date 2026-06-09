<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="min-h-screen w-full flex flex-col items-center justify-center p-4 md:p-8 selection:bg-sky-500/30">
  <div class="relative w-full max-w-lg p-8 md:p-10 rounded-3xl bg-white/70 dark:bg-slate-900/60 backdrop-blur-xl border border-white/20 dark:border-slate-800/40 shadow-2xl flex flex-col items-center text-center transition-all duration-300">
    
    <!-- Background glow accents -->
    <div class="absolute -top-12 -left-12 w-48 h-48 bg-sky-400/20 dark:bg-sky-500/10 rounded-full blur-3xl pointer-events-none"></div>
    <div class="absolute -bottom-12 -right-12 w-48 h-48 bg-violet-400/20 dark:bg-indigo-500/10 rounded-full blur-3xl pointer-events-none"></div>

    <!-- Logos Section -->
    <div class="flex items-center justify-center gap-6 mb-8 relative z-10">
      <a href="https://vite.dev" target="_blank" class="group transition-transform duration-300 hover:scale-110">
        <img src="/vite.svg" class="h-16 w-16 drop-shadow-sm group-hover:drop-shadow-[0_0_20px_rgba(116,123,255,0.6)] transition-all duration-500" alt="Vite Logo" />
      </a>
      <a href="https://tauri.app" target="_blank" class="group transition-transform duration-300 hover:scale-110">
        <img src="/tauri.svg" class="h-16 w-16 drop-shadow-sm group-hover:drop-shadow-[0_0_20px_rgba(36,200,219,0.6)] transition-all duration-500" alt="Tauri Logo" />
      </a>
      <a href="https://svelte.dev" target="_blank" class="group transition-transform duration-300 hover:scale-110">
        <img src="/svelte.svg" class="h-16 w-16 drop-shadow-sm group-hover:drop-shadow-[0_0_20px_rgba(255,62,0,0.6)] transition-all duration-500" alt="SvelteKit Logo" />
      </a>
    </div>

    <!-- Typography -->
    <h1 class="text-3xl md:text-4xl font-extrabold tracking-tight bg-gradient-to-r from-sky-500 via-indigo-500 to-purple-600 dark:from-sky-400 dark:via-indigo-400 dark:to-purple-500 bg-clip-text text-transparent mb-3 relative z-10">
      Welcome to Tauri + Svelte
    </h1>
    
    <p class="text-sm md:text-base text-slate-500 dark:text-slate-400 max-w-sm mb-8 leading-relaxed relative z-10">
      Click on the Tauri, Vite, and SvelteKit logos to learn more.
    </p>

    <!-- Interactive Form -->
    <form class="w-full flex flex-col sm:flex-row gap-3 mb-6 relative z-10" onsubmit={greet}>
      <input 
        id="greet-input" 
        type="text"
        placeholder="Enter a name..." 
        bind:value={name} 
        class="flex-1 px-4 py-3 rounded-xl bg-white/80 dark:bg-slate-950/80 border border-slate-200 dark:border-slate-800 text-slate-900 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-sky-500/50 focus:border-sky-500 transition-all duration-200 text-sm shadow-inner"
      />
      <button 
        type="submit"
        class="px-6 py-3 rounded-xl bg-gradient-to-r from-sky-500 to-indigo-600 hover:from-sky-600 hover:to-indigo-700 text-white font-semibold text-sm shadow-md hover:shadow-lg hover:shadow-indigo-500/20 active:scale-[0.98] transition-all duration-200 cursor-pointer"
      >
        Greet
      </button>
    </form>

    <!-- Output Box -->
    <div class="h-12 flex items-center justify-center relative z-10">
      {#if greetMsg}
        <div class="px-4 py-2 rounded-xl bg-sky-500/10 dark:bg-sky-500/5 border border-sky-500/20 text-sky-700 dark:text-sky-300 text-sm font-medium animate-fade-in">
          {greetMsg}
        </div>
      {/if}
    </div>
  </div>
</main>

