<script setup lang="ts">
import { LineChart, type LineSeriesOption } from "echarts/charts";
import { GridComponent, MarkAreaComponent, TooltipComponent } from "echarts/components";
import { init, use, type ECharts } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { t } from "@i18n";

use([CanvasRenderer, GridComponent, LineChart, MarkAreaComponent, TooltipComponent]);

const props = defineProps<{
  rttMs: number | null;
}>();

type Sample = { time: number; value: number };

const MAX_SAMPLES = 60;
const WARMUP_DELAY_MS = 3000;
const LOW_LATENCY_MAX_MS = 80;
const MEDIUM_LATENCY_MAX_MS = 180;
const MIN_CHART_MAX_MS = 250;
const chartElement = ref<HTMLDivElement | null>(null);
const currentLatency = computed(() => (props.rttMs == null ? "--" : `${props.rttMs} ms`));
const samples: Sample[] = [];
let chart: ECharts | null = null;
let timer: number | null = null;
let warmupTimer: number | null = null;
let resizeObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;

function cssVariable(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function updateChart() {
  if (!chart) return;
  const primary = cssVariable("--primary-solid") || cssVariable("--primary");
  const border = cssVariable("--border");
  const muted = cssVariable("--muted");
  const surface = cssVariable("--surface");
  const success = cssVariable("--success");
  const warning = cssVariable("--warning");
  const danger = cssVariable("--danger");
  const values = samples.map((sample) => sample.value);
  const latencyPadding = Math.max(
    5,
    Math.ceil((Math.max(...values, 0) - Math.min(...values, 0)) * 0.25),
  );
  const chartMax = Math.max(MIN_CHART_MAX_MS, Math.max(...values, 0) + latencyPadding);
  const series: LineSeriesOption = {
    type: "line",
    data: samples.map((sample) => [sample.time, sample.value]),
    smooth: 0.35,
    showSymbol: false,
    symbol: "circle",
    lineStyle: { color: primary, width: 2 },
    itemStyle: { color: primary },
    areaStyle: { color: primary, opacity: 0.06 },
    markArea: {
      silent: true,
      label: { position: "insideRight", color: muted, fontSize: 10 },
      data: [
        [
          {
            name: t("join.lowLatency", { threshold: LOW_LATENCY_MAX_MS }),
            yAxis: 0,
            itemStyle: { color: success, opacity: 0.12 },
          },
          { yAxis: LOW_LATENCY_MAX_MS },
        ],
        [
          {
            name: t("join.mediumLatency", {
              low: LOW_LATENCY_MAX_MS,
              high: MEDIUM_LATENCY_MAX_MS,
            }),
            yAxis: LOW_LATENCY_MAX_MS,
            itemStyle: { color: warning, opacity: 0.12 },
          },
          { yAxis: MEDIUM_LATENCY_MAX_MS },
        ],
        [
          {
            name: t("join.highLatency", { threshold: MEDIUM_LATENCY_MAX_MS }),
            yAxis: MEDIUM_LATENCY_MAX_MS,
            itemStyle: { color: danger, opacity: 0.1 },
          },
          { yAxis: chartMax },
        ],
      ],
    },
    emphasis: { focus: "series" },
  };
  chart.setOption({
    animation: !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    animationDuration: 220,
    animationDurationUpdate: 260,
    animationEasingUpdate: "cubicOut",
    grid: { top: 14, right: 12, bottom: 18, left: 42 },
    tooltip: {
      trigger: "axis",
      backgroundColor: surface,
      borderColor: border,
      borderWidth: 1,
      textStyle: { color: cssVariable("--text"), fontSize: 12 },
      valueFormatter: (value: number | string) => `${value} ms`,
    },
    xAxis: {
      type: "time",
      boundaryGap: false,
      axisLine: { lineStyle: { color: border } },
      axisTick: { show: false },
      axisLabel: { show: false },
      splitLine: { show: false },
    },
    yAxis: {
      type: "value",
      min: 0,
      max: chartMax,
      axisLabel: { color: muted, fontSize: 11, formatter: "{value} ms" },
      axisLine: { show: false },
      axisTick: { show: false },
      splitLine: { lineStyle: { color: border, type: "dashed" } },
    },
    series: [series],
  });
}

function sampleLatency() {
  if (props.rttMs == null) return;
  samples.push({ time: Date.now(), value: props.rttMs });
  if (samples.length > MAX_SAMPLES) samples.shift();
  updateChart();
}

onMounted(() => {
  if (!chartElement.value) return;
  chart = init(chartElement.value, undefined, { renderer: "canvas" });
  updateChart();
  warmupTimer = window.setTimeout(() => {
    sampleLatency();
    timer = window.setInterval(sampleLatency, 1000);
  }, WARMUP_DELAY_MS);
  resizeObserver = new ResizeObserver(() => chart?.resize());
  resizeObserver.observe(chartElement.value);
  themeObserver = new MutationObserver(updateChart);
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class", "data-theme", "style"],
  });
});

onUnmounted(() => {
  if (timer != null) window.clearInterval(timer);
  if (warmupTimer != null) window.clearTimeout(warmupTimer);
  resizeObserver?.disconnect();
  themeObserver?.disconnect();
  chart?.dispose();
});
</script>

<template>
  <section class="latency-chart" :aria-label="t('join.latencyHistory')">
    <div class="latency-chart-heading">
      <span>{{ t("join.latencyHistory") }}</span>
      <strong>{{ currentLatency }}</strong>
    </div>
    <div ref="chartElement" class="latency-chart-canvas" />
  </section>
</template>
