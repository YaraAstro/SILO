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

<nav class="fixed inset-x-0 bottom-4 z-40 flex justify-center px-4">
  <div class="flex max-w-full gap-1 overflow-x-auto rounded-xl border border-slate-700/80 bg-slate-900/95 p-2 shadow-2xl shadow-black/35 backdrop-blur">
    {#each items as item}
      <button
        class="inline-flex min-h-11 shrink-0 items-center gap-2 rounded-lg px-3 text-sm font-semibold outline-none transition focus-visible:ring-2 focus-visible:ring-teal-300 sm:px-5 {active === item.key
          ? 'bg-teal-400 text-slate-950'
          : 'text-slate-400 hover:bg-slate-800 hover:text-slate-100'}"
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
