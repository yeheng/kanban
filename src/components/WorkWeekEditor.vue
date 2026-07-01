<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { useCalendarStore } from "@/stores/calendar";
const cal = useCalendarStore();
const labels = ["一", "二", "三", "四", "五", "六", "日"];

function cycle(i: number) {
  const cur = cal.week[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...cal.week]; w[i] = next; cal.setWeek(w);
}
function dayType(f: number): "default" | "secondary" | "outline" {
  if (f === 0) return "outline";
  if (f === 0.5) return "secondary";
  return "default";
}
</script>

<template>
  <div>
    <div class="flex gap-1">
      <Button
        v-for="(f, i) in cal.week"
        :key="i"
        :variant="dayType(f)"
        size="sm"
        @click="cycle(i)"
      >
        {{ labels[i] }}
        <span class="work-week-editor__suffix">
          {{ f === 0 ? "休" : f === 0.5 ? "半" : "全" }}
        </span>
      </Button>
    </div>
    <span class="block text-xs text-muted-foreground mt-1.5">
      点击切换 全天/半天/休息（写入全局工作周模板）
    </span>
  </div>
</template>

<style scoped>
.work-week-editor__suffix {
  font-size: 10px;
  margin-left: 2px;
}
</style>
