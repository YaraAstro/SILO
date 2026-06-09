<script lang="ts">
  import { onMount } from "svelte";
  import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Filler,
    Legend,
    LinearScale,
    LineController,
    LineElement,
    PointElement,
    Tooltip,
    type ChartDataset,
    type ChartType
  } from "chart.js";

  Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    Filler,
    Legend,
    LinearScale,
    LineController,
    LineElement,
    PointElement,
    Tooltip
  );

  let {
    labels,
    datasets,
    type = "bar",
    height = 260
  }: {
    labels: string[];
    datasets: ChartDataset[];
    type?: ChartType;
    height?: number;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let chart: Chart | null = null;
  let mounted = false;

  function renderChart() {
    if (!canvas || !mounted) return;
    chart?.destroy();
    chart = new Chart(canvas, {
      type,
      data: { labels, datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        interaction: { intersect: false, mode: "index" },
        plugins: {
          legend: {
            display: datasets.length > 1,
            labels: { color: "#94a3b8", boxWidth: 8, boxHeight: 8, usePointStyle: true }
          },
          tooltip: {
            backgroundColor: "#020617",
            borderColor: "#334155",
            borderWidth: 1,
            titleColor: "#f8fafc",
            bodyColor: "#cbd5e1"
          }
        },
        scales: {
          x: {
            grid: { display: false },
            ticks: { color: "#64748b" }
          },
          y: {
            beginAtZero: true,
            grid: { color: "rgba(51, 65, 85, 0.45)" },
            ticks: { color: "#64748b" }
          }
        }
      }
    });
  }

  onMount(() => {
    mounted = true;
    renderChart();
    return () => chart?.destroy();
  });

  $effect(() => {
    labels;
    datasets;
    type;
    renderChart();
  });
</script>

<div style={`height: ${height}px`}>
  <canvas bind:this={canvas}></canvas>
</div>
