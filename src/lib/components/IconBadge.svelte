<script lang="ts">
  let {
    icon: Icon,
    tone = "teal",
    label = "",
    size = "md"
  }: {
    icon: any;
    tone?: "teal" | "purple" | "yellow" | "red" | "blue" | "neutral";
    label?: string;
    size?: "sm" | "md" | "lg";
  } = $props();

  const toneClass = {
    teal: "bg-teal-500/15 text-teal-300",
    purple: "bg-violet-500/15 text-violet-300",
    yellow: "bg-amber-500/15 text-amber-300",
    red: "bg-red-500/15 text-red-300",
    blue: "bg-blue-500/15 text-blue-300",
    neutral: "bg-slate-700 text-slate-300"
  };

  const sizeClass = {
    sm: "h-8 w-8",
    md: "h-10 w-10",
    lg: "h-12 w-12"
  };

  const iconSize = {
    sm: 16,
    md: 19,
    lg: 22
  };
</script>

<span class="inline-flex shrink-0 items-center justify-center rounded-lg {toneClass[tone]} {sizeClass[size]}" aria-label={label || undefined}>
  <Icon size={iconSize[size]} strokeWidth={2.2} />
</span>
