<script lang="ts">
  import IconBadge from "./IconBadge.svelte";

  let {
    title,
    value,
    caption = "",
    trend = "",
    icon,
    tone = "teal"
  }: {
    title: string;
    value: string;
    caption?: string;
    trend?: string;
    icon: any;
    tone?: "teal" | "purple" | "yellow" | "red" | "blue" | "neutral";
  } = $props();
</script>

<section class="silo-card p-5">
  <IconBadge {icon} {tone} label={title} />
  <p class="mt-4 text-2xl font-bold leading-none text-slate-100">{value}</p>
  <p class="mt-2 text-sm text-slate-400">{title}</p>
  {#if caption || trend}
    <p class="mt-3 text-xs {trend ? 'text-emerald-300' : 'text-slate-500'}">{trend || caption}</p>
  {/if}
</section>
