<script lang="ts">
  let {
    checked = false,
    disabled = false,
    label,
    onchange
  }: {
    checked?: boolean;
    disabled?: boolean;
    label: string;
    onchange: (checked: boolean) => void;
  } = $props();
</script>

<button
  class="relative inline-flex h-6 w-11 shrink-0 items-center rounded-full outline-none transition focus-visible:ring-2 focus-visible:ring-teal-300 disabled:cursor-not-allowed disabled:opacity-50 {checked ? 'bg-teal-400' : 'bg-slate-950'}"
  type="button"
  role="switch"
  aria-label={label}
  aria-checked={checked}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="h-5 w-5 rounded-full bg-slate-100 shadow transition {checked ? 'translate-x-5' : 'translate-x-0.5'}"></span>
</button>
