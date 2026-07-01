<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { api } from "@/api";
import { useResourcesStore } from "@/stores/resources";
import { useWorkloadStore } from "@/stores/workload";
import { useRefreshStore } from "@/stores/refresh";
import OccupancyGrid from "@/components/OccupancyGrid.vue";
import { fmtDate, parseDateStrict } from "@/utils/date";
import type { DayOccupancy } from "@/types";

const resources = useResourcesStore();
const wl = useWorkloadStore();
const refreshBus = useRefreshStore();
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const items = ref<DayOccupancy[]>([]);
const days = ref<string[]>([]);

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
// Reload occupancy when an allocation write / AI accept bumps the refresh bus (design G4).
watch(() => refreshBus.version.calendar, () => { void refresh(); });
</script>

<template>
  <h2 class="text-2xl font-bold mt-0">日历 / Calendar 占用</h2>
  <div class="flex items-center gap-2 mb-3">
    <DateRangePicker v-model="dateRange" />
    <Button @click="refresh">刷新</Button>
  </div>
  <OccupancyGrid :items="items" :days="days" :resources="resources.items" />
</template>
