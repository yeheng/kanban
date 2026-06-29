<script setup lang="ts">
import { NSpace, NButton, NText } from "naive-ui";
import { useCalendarStore } from "../stores/calendar";
const cal = useCalendarStore();
const labels = ["一", "二", "三", "四", "五", "六", "日"];

function cycle(i: number) {
  const cur = cal.week[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...cal.week]; w[i] = next; cal.setWeek(w);
}
function dayType(f: number): "default" | "info" | "success" {
  if (f === 0) return "default";
  if (f === 0.5) return "info";
  return "success";
}
</script>

<template>
  <div>
    <n-space :size="4">
      <n-button
        v-for="(f, i) in cal.week"
        :key="i"
        :type="dayType(f)"
        size="small"
        @click="cycle(i)"
      >
        {{ labels[i] }}
        <span class="work-week-editor__suffix">
          {{ f === 0 ? "休" : f === 0.5 ? "半" : "全" }}
        </span>
      </n-button>
    </n-space>
    <n-text depth="3" style="display: block; font-size: 12px; margin-top: 6px">
      点击切换 全天/半天/休息（写入全局工作周模板）
    </n-text>
  </div>
</template>

<style scoped>
.work-week-editor__suffix {
  font-size: 10px;
  margin-left: 2px;
}
</style>
