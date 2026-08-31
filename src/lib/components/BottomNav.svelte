<script lang="ts">
  type NavItem<T extends string> = {
    key: T;
    label: string;
    icon: any;
  };

  let {
    items,
    active,
    onSelect
  }: {
    items: Array<NavItem<string>>;
    active: string;
    onSelect: (key: string) => void;
  } = $props();
</script>

<nav class="fixed inset-x-0 bottom-4 z-40 flex justify-center px-3 sm:px-4">
  <div class="flex max-w-full gap-1 overflow-x-auto rounded-2xl border border-slate-700/70 bg-slate-950/90 p-1.5 shadow-2xl shadow-black/40 backdrop-blur-xl">
    {#each items as item}
      <button
        class="inline-flex min-h-11 shrink-0 items-center gap-2 rounded-xl px-3 text-sm font-semibold outline-none transition-colors duration-150 focus-visible:ring-2 focus-visible:ring-teal-300 sm:px-5 {active === item.key
          ? 'bg-teal-300 text-slate-950 shadow-lg shadow-teal-500/10'
          : 'text-slate-400 hover:bg-slate-800/80 hover:text-slate-100'}"
        type="button"
        aria-current={active === item.key ? "page" : undefined}
        onclick={() => onSelect(item.key)}
      >
        <item.icon size={18} strokeWidth={2.2} />
        <span>{item.label}</span>
      </button>
    {/each}
  </div>
</nav>
