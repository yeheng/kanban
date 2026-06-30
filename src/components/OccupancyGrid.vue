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
  // Prefer the server's per-team band; fall back to the global band for older payloads.
  const b = o.status ?? wl.band(o.utilization);
  return ({ under: "#e0e0e6", green: "#9ad19a", yellow: "#f0d070", red: "#e08090" } as const)[b];
}
</script>

<template>
  <div class="occupancy-grid">
    <table>
      <thead>
        <tr>
          <th>资源</th>
          <th v-for="d in days" :key="d">{{ d.slice(8) }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="r in resources" :key="r.id">
          <td class="occupancy-grid__res">{{ r.name }}</td>
          <td
            v-for="d in days"
            :key="d"
            :style="{ background: bg(cell(r.id, d)) }"
          >
            <span v-if="cell(r.id, d)" class="occupancy-grid__pct">
              {{ Math.round(cell(r.id, d)!.utilization * 100) }}%
            </span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.occupancy-grid {
  overflow-x: auto;
}
.occupancy-grid table {
  border-collapse: collapse;
  font-size: 12px;
}
.occupancy-grid th,
.occupancy-grid td {
  border: 1px solid #e0e0e6;
  padding: 4px 6px;
  text-align: center;
  white-space: nowrap;
}
.occupancy-grid__res {
  text-align: left;
  font-weight: 500;
  position: sticky;
  left: 0;
  background: #fff;
  z-index: 1;
}
.occupancy-grid__pct {
  font-size: 11px;
}
</style>
