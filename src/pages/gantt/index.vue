<script setup lang="ts">
import { computed, ref, watchEffect } from "vue";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useGanttStore } from "@/stores/gantt";
import { useProjectsStore } from "@/stores/projects";
import { useResourcesStore } from "@/stores/resources";
import { useRefreshStore } from "@/stores/refresh";
import GanttTimeline from "@/components/GanttTimeline.vue";

const gantt = useGanttStore();
const projects = useProjectsStore();
const resources = useResourcesStore();
const refreshBus = useRefreshStore();
const err = ref<string | null>(null);
const start = ref("2026-06-29");
const end = ref("2026-08-09");
const resourceSelect = ref<number | null>(null);

const projectOptions = computed(() =>
  projects.items.map((p) => ({ label: p.name, value: p.id })),
);
const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);

watchEffect(async () => {
  // Reload when the refresh bus bumps gantt (e.g. after an allocation write / AI accept).
  void refreshBus.version.gantt;
  if (gantt.mode === "project" && projects.current != null) {
    gantt.focusId = projects.current;
    await safeLoad();
  }
});
async function safeLoad() {
  try { err.value = null; await gantt.load(); } catch (e: unknown) { err.value = e instanceof Error ? e.message : String(e); }
}
async function onProjectChange(value: unknown) {
  const id = value as number | undefined;
  if (id != null) projects.select(id);
  await safeLoad();
}
async function onResource(value: unknown) {
  const id = value as number | undefined;
  if (id == null) return;
  gantt.mode = "resource";
  gantt.focusId = id;
  await safeLoad();
}
async function toProjectMode() { gantt.mode = "project"; await safeLoad(); }
</script>

<template>
  <h2 class="text-2xl font-bold mt-0">Gantt</h2>
  <div class="flex items-center gap-2 mb-2">
    <span>模式：</span>
    <Button :disabled="gantt.mode === 'project'" @click="toProjectMode">项目</Button>
    <Select
      :model-value="projects.current ?? undefined"
      :disabled="gantt.mode !== 'project'"
      @update:model-value="onProjectChange"
    >
      <SelectTrigger class="w-[200px]">
        <SelectValue placeholder="选择项目" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="opt in projectOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </SelectItem>
      </SelectContent>
    </Select>
    <span>或资源视角：</span>
    <Select
      :model-value="resourceSelect ?? undefined"
      @update:model-value="onResource"
    >
      <SelectTrigger class="w-[200px]">
        <SelectValue placeholder="选择资源" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="opt in resourceOptions"
          :key="opt.value"
          :value="opt.value"
        >
          {{ opt.label }}
        </SelectItem>
      </SelectContent>
    </Select>
    <Alert v-if="err" variant="destructive" class="py-1 px-2 w-auto">
      <AlertDescription>⚠ {{ err }}（可能越出任务/资源时间窗）</AlertDescription>
    </Alert>
  </div>
  <GanttTimeline :start="start" :end="end" />
</template>
