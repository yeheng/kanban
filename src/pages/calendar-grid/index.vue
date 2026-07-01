<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useDailyOccupancyQuery } from "@/services/api/gantt.api";
import OccupancyGrid from "@/components/OccupancyGrid.vue";
import { fmtDate, parseDateStrict } from "@/utils/date";

const resourcesQuery = useListResourcesQuery();
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const days = ref<string[]>([]);

const startStr = computed(() => fmtDate(dateRange.value[0]));
const endStr = computed(() => fmtDate(dateRange.value[1]));
const occupancyQuery = useDailyOccupancyQuery(startStr, endStr);

function buildDays() {
  const out: string[] = [];
  const d0 = new Date(fmtDate(dateRange.value[0]) + "T00:00:00");
  const d1 = new Date(fmtDate(dateRange.value[1]) + "T00:00:00");
  for (let d = new Date(d0); d <= d1; d.setDate(d.getDate() + 1)) {
    out.push(fmtDate(d.getTime()));
  }
  days.value = out;
}

function refresh() {
  buildDays();
  occupancyQuery.refetch();
}

watch(dateRange, () => {
  buildDays();
}, { immediate: true });
</script>

<template>
  <h2 class="text-2xl font-bold mt-0">日历 / Calendar 占用</h2>
  <div class="flex items-center gap-2 mb-3">
    <DateRangePicker v-model="dateRange" />
    <Button @click="refresh">刷新</Button>
  </div>
  <OccupancyGrid :items="occupancyQuery.data.value ?? []" :days="days" :resources="resourcesQuery.data.value ?? []" />
</template>
