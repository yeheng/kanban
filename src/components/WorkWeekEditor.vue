<script setup lang="ts">
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { useListWorkWeeksQuery, useSetGlobalWorkWeekMutation } from "@/services/api/calendar.api";

const weekQuery = useListWorkWeeksQuery();
const setWeekMutation = useSetGlobalWorkWeekMutation();

const labels = ["一", "二", "三", "四", "五", "六", "日"];

const week = computed(() => {
  const rows = weekQuery.data.value ?? [];
  const global = rows.find((r) => r.scope === "global");
  if (!global) return [1, 1, 1, 1, 1, 0, 0];
  const f = (bit: number, frac: number) => (bit ? frac : 0);
  return [
    f(global.mon, global.mon_frac),
    f(global.tue, global.tue_frac),
    f(global.wed, global.wed_frac),
    f(global.thu, global.thu_frac),
    f(global.fri, global.fri_frac),
    f(global.sat, global.sat_frac),
    f(global.sun, global.sun_frac),
  ];
});

async function cycle(i: number) {
  const cur = week.value[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...week.value];
  w[i] = next;
  await setWeekMutation.mutateAsync(w);
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
        v-for="(f, i) in week"
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
