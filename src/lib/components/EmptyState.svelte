<script lang="ts">
  let {
    icon: Icon,
    title,
    message,
    compact = false
  }: {
    icon?: any;
    title: string;
    message: string;
    compact?: boolean;
  } = $props();
</script>

<div class="rounded-lg border border-dashed border-slate-700 bg-slate-950/35 text-center {compact ? 'p-4' : 'p-8'}">
  {#if Icon}
    <div class="mx-auto flex h-10 w-10 items-center justify-center rounded-lg bg-slate-800 text-slate-400">
      <Icon size={18} strokeWidth={2.2} />
    </div>
  {/if}
  <p class="{Icon ? 'mt-3' : ''} text-sm font-semibold text-slate-200">{title}</p>
  <p class="mx-auto mt-1 max-w-md text-sm text-slate-500">{message}</p>
</div>
