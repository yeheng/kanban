<script setup lang="ts">
import { ref } from "vue";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { api } from "../api";
import type { Task } from "../types";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const resourceId = ref<number | null>(null);
const taskId = ref<number | null>(null);
const start = ref("2026-06-29"); const end = ref("2026-07-03"); const percent = ref(0.5);
const tasks = ref<Task[]>([]);
const impact = ref<{ utilization: number; overloaded: boolean } | null>(null);
const error = ref<string | null>(null);

async function loadTasks() {
  if (projects.current == null) return;
  tasks.value = await api.listTasks(projects.current);
}
async function submit() {
  error.value = null;
  if (resourceId.value == null || taskId.value == null || projects.current == null) return;
  try {
    await allocations.create(resourceId.value, taskId.value, start.value, end.value, percent.value);
    await allocations.load(projects.current);
    const s = await api.resourceSummary(resourceId.value, start.value, end.value);
    impact.value = { utilization: s.utilization, overloaded: s.overloaded };
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}
resources.load();
loadTasks();
</script>
<template>
  <form @submit.prevent="submit">
    <select v-model.number="resourceId"><option :value="null">资源</option><option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option></select>
    <select v-model.number="taskId"><option :value="null">任务</option><option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.title }}</option></select>
    <input v-model="start" type="date" /><input v-model="end" type="date" />
    <input v-model.number="percent" type="number" min="0.01" max="1" step="0.05" />
    <button>分配</button>
    <span v-if="error" style="color:#d03050">{{ error }}</span>
    <span v-if="impact" :style="{ color: impact.overloaded ? '#d03050' : '#18a058' }">→ 利用率 {{ Math.round(impact.utilization * 100) }}%{{ impact.overloaded ? ' ⚠过载' : '' }}</span>
  </form>
</template>
