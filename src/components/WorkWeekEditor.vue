<script setup lang="ts">
import { useCalendarStore } from "../stores/calendar";
const cal = useCalendarStore();
const labels = ["一", "二", "三", "四", "五", "六", "日"];
function cycle(i: number) {
  const cur = cal.week[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...cal.week]; w[i] = next; cal.setWeek(w);
}
</script>
<template>
  <div>
    <span v-for="(f, i) in cal.week" :key="i" @click="cycle(i)" class="day" :style="{ opacity: f === 0 ? 0.3 : 1 }">
      {{ labels[i] }}<small>{{ f === 0 ? "休" : f === 0.5 ? "半" : "全" }}</small>
    </span>
    <p><small>点击切换 全天/半天/休息（写入全局工作周模板）</small></p>
  </div>
</template>
<style scoped>
.day { display: inline-block; cursor: pointer; border: 1px solid #ccc; border-radius: 6px; padding: 6px 10px; margin: 2px; user-select: none; }
small { display: block; font-size: 10px; color: #888; }
</style>
