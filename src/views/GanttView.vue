<script setup lang="ts">
import { computed, ref, watchEffect } from "vue";
import { NH2, NSpace, NButton, NSelect, NText, NAlert } from "naive-ui";
import { useGanttStore } from "../stores/gantt";
import { useProjectsStore } from "../stores/projects";
import { useResourcesStore } from "../stores/resources";
import GanttTimeline from "../components/GanttTimeline.vue";

const gantt = useGanttStore();
const projects = useProjectsStore();
const resources = useResourcesStore();
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
  if (gantt.mode === "project" && projects.current != null) {
    gantt.focusId = projects.current;
    await safeLoad();
  }
});
async function safeLoad() {
  try { err.value = null; await gantt.load(); } catch (e: unknown) { err.value = e instanceof Error ? e.message : String(e); }
}
async function onProjectChange(id: number | null) {
  if (id != null) projects.select(id);
  await safeLoad();
}
async function onResource(id: number | null) {
  if (id == null) return;
  gantt.mode = "resource";
  gantt.focusId = id;
  await safeLoad();
}
async function toProjectMode() { gantt.mode = "project"; await safeLoad(); }
</script>

<template>
  <n-h2 style="margin-top: 0">Gantt</n-h2>
  <n-space align="center" :size="8" style="margin-bottom: 8px">
    <span>模式：</span>
    <n-button :disabled="gantt.mode === 'project'" @click="toProjectMode">项目</n-button>
    <n-select
      :value="projects.current"
      :options="projectOptions"
      :disabled="gantt.mode !== 'project'"
      placeholder="选择项目"
      style="width: 200px"
      @update:value="onProjectChange"
    />
    <span>或资源视角：</span>
    <n-select
      v-model:value="resourceSelect"
      :options="resourceOptions"
      placeholder="选择资源"
      style="width: 200px"
      @update:value="onResource"
    />
    <n-text v-if="err" type="error">⚠ {{ err }}（可能越出任务/资源时间窗）</n-text>
  </n-space>
  <GanttTimeline :start="start" :end="end" />
</template>
