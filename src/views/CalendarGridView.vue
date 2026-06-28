<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";
import { useResourcesStore } from "../stores/resources";
import { useWorkloadStore } from "../stores/workload";
import OccupancyGrid from "../components/OccupancyGrid.vue";
import type { DayOccupancy } from "../types";

const resources = useResourcesStore();
const wl = useWorkloadStore();
const start = ref("2026-06-29"); const end = ref("2026-07-12");
const items = ref<DayOccupancy[]>([]);
const days = ref<string[]>([]);

function buildDays() {
  const out: string[] = []; let ms = Date.parse(start.value);
  const endMs = Date.parse(end.value);
  while (ms <= endMs) { out.push(toStr(ms)); ms += 86400000; }
  days.value = out;
}
function toStr(ms: number) {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
async function refresh() {
  buildDays();
  items.value = await api.dailyOccupancy(start.value, end.value);
}
onMounted(async () => { await wl.loadThresholds(); await resources.load(); await refresh(); });
</script>
<template>
  <h2 style="margin-top:0">日历 / Calendar 占用</h2>
  <input v-model="start" type="date" /> – <input v-model="end" type="date" />
  <button @click="refresh">刷新</button>
  <OccupancyGrid :items="items" :days="days" :resources="resources.items" />
</template>
