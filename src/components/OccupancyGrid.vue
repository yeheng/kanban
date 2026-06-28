<script setup lang="ts">
import { useWorkloadStore } from "../stores/workload";
import type { DayOccupancy } from "../types";
const props = defineProps<{ items: DayOccupancy[]; days: string[]; resources: { id: number; name: string }[] }>();
const wl = useWorkloadStore();
function cell(rid: number, day: string) {
  return props.items.find((o) => o.resource_id === rid && o.date === day);
}
function bg(o?: DayOccupancy) {
  if (!o) return "#f7f7fa";
  return { under: "#e0e0e6", green: "#9ad19a", yellow: "#f0d070", red: "#e08090" }[wl.band(o.utilization)];
}
</script>
<template>
  <table border="1" cellpadding="6" style="border-collapse:collapse">
    <thead><tr><th>资源</th><th v-for="d in days" :key="d">{{ d.slice(8) }}</th></tr></thead>
    <tbody>
      <tr v-for="r in resources" :key="r.id">
        <td>{{ r.name }}</td>
        <td v-for="d in days" :key="d" :style="{ background: bg(cell(r.id, d)) }">
          <small v-if="cell(r.id, d)">{{ Math.round(cell(r.id, d)!.utilization * 100) }}%</small>
        </td>
      </tr>
    </tbody>
  </table>
</template>
