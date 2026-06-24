<script lang="ts">
  let {
    value = $bindable(""),
    length = 6,
    onsubmit = () => {},
    oncancel = () => {},
  } = $props<{
    value: string;
    length?: number;
    onsubmit?: () => void;
    oncancel?: () => void;
  }>();

  let inputEl: HTMLInputElement;

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    target.value = target.value.replace(/[^0-9]/g, "").slice(0, length);
    value = target.value;
    if (value.length === length) {
      onsubmit();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" && value.length === length) {
      onsubmit();
    } else if (e.key === "Escape") {
      oncancel();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="relative w-full cursor-text group" onclick={() => inputEl?.focus()}>
  <input
    bind:this={inputEl}
    type="text"
    inputmode="numeric"
    class="absolute inset-0 w-full h-full opacity-0 cursor-text z-10"
    bind:value
    oninput={handleInput}
    onkeydown={handleKeyDown}
    autofocus
  />
  <div class="flex gap-2.5 justify-center pointer-events-none">
    {#each Array(length) as _, i}
      <div class="w-12 h-14 rounded-xl border flex items-center justify-center text-2xl font-black font-mono transition-all duration-300 shadow-md relative overflow-hidden
        {value.length > i 
          ? 'border-teal-500/60 bg-teal-500/10 text-teal-300 scale-105' 
          : value.length === i 
            ? 'border-purple-500/60 bg-purple-500/5 text-purple-300 scale-[1.02] ring-1 ring-purple-500/30' 
            : 'border-slate-800 bg-slate-950/50 text-slate-600'}"
      >
        {#if value.length === i}
          <div class="absolute inset-x-0 bottom-0 h-1 bg-purple-400 animate-pulse"></div>
        {/if}
        {#if value.length > i}•{/if}
      </div>
    {/each}
  </div>
</div>
