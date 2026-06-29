<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NH2, NSpace, NDatePicker, NButton } from "naive-ui";
import { api } from "../api";
import { useResourcesStore } from "../stores/resources";
import { useWorkloadStore } from "../stores/workload";
import OccupancyGrid from "../components/OccupancyGrid.vue";
import type { DayOccupancy } from "../types";

const resources = useResourcesStore();
const wl = useWorkloadStore();
const dateRange = ref<[number, number]>([Date.parse("2026-06-29"), Date.parse("2026-07-12")]);
const items = ref<DayOccupancy[]>([]);
const days = ref<string[]>([]);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function buildDays() {
  const out: string[] = [];
  const d0 = new Date(fmtDate(dateRange.value[0]) + "T00:00:00");
  const d1 = new Date(fmtDate(dateRange.value[1]) + "T00:00:00");
  for (let d = new Date(d0); d <= d1; d.setDate(d.getDate() + 1)) {
    out.push(fmtDate(d.getTime()));
  }
  days.value = out;
}
async function refresh() {
  buildDays();
  const start = fmtDate(dateRange.value[0]);
  const end = fmtDate(dateRange.value[1]);
  items.value = await api.dailyOccupancy(start, end);
}
onMounted(async () => { await wl.loadThresholds(); await resources.load(); await refresh(); });
</script>

<template>
  <n-h2 style="margin-top: 0">日历 / Calendar 占用</n-h2>
  <n-space align="center" :size="8" style="margin-bottom: 12px">
    <n-date-picker v-model:value="dateRange" type="daterange" clearable />
    <n-button type="primary" @click="refresh">刷新</n-button>
  </n-space>
  <OccupancyGrid :items="items" :days="days" :resources="resources.items" />
</template>
