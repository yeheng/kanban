<script setup lang="ts">
import { computed } from "vue";
import { useWorkloadStore } from "../stores/workload";
const props = defineProps<{ utilization: number }>();
const wl = useWorkloadStore();
const pct = computed(() => Math.min(150, Math.round(props.utilization * 100)));
const band = computed(() => wl.band(props.utilization));
const color = computed(() => ({ under: "#d1d1d6", green: "#18a058", yellow: "#f0a020", red: "#d03050" }[band.value]));
</script>
<template>
  <div class="wrap" :title="`${pct}% (${band})`">
    <div class="fill" :style="{ width: pct + '%', background: color }" />
    <span class="label">{{ pct }}%</span>
  </div>
</template>
<style scoped>
.wrap { position: relative; width: 160px; height: 18px; background: #f0f0f0; border-radius: 4px; overflow: hidden; }
.fill { height: 100%; }
.label { position: absolute; right: 6px; top: 0; font-size: 11px; line-height: 18px; color: #333; }
</style>
