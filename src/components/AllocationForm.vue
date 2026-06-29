<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NForm, NFormItem, NSelect, NDatePicker, NInputNumber, NButton, NTag, NText } from "naive-ui";
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
const dateRange = ref<[number, number]>([Date.parse("2026-06-29"), Date.parse("2026-07-03")]);
const percent = ref(0.5);
const tasks = ref<Task[]>([]);
const impact = ref<{ utilization: number; overloaded: boolean } | null>(null);
const error = ref<string | null>(null);

const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);
const taskOptions = computed(() =>
  tasks.value.map((t) => ({ label: t.title, value: t.id })),
);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function loadTasks() {
  if (projects.current == null) return;
  tasks.value = await api.listTasks(projects.current);
}
async function submit() {
  error.value = null;
  if (resourceId.value == null || taskId.value == null || projects.current == null) return;
  const start = fmtDate(dateRange.value[0]);
  const end = fmtDate(dateRange.value[1]);
  try {
    await allocations.create(resourceId.value, taskId.value, start, end, percent.value);
    await allocations.load(projects.current);
    const s = await api.resourceSummary(resourceId.value, start, end);
    impact.value = { utilization: s.utilization, overloaded: s.overloaded };
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}
resources.load();
watch(() => projects.current, () => { loadTasks(); }, { immediate: true });
</script>

<template>
  <n-form inline @submit.prevent="submit">
    <n-form-item label="资源">
      <n-select v-model:value="resourceId" :options="resourceOptions" placeholder="选择资源" />
    </n-form-item>
    <n-form-item label="任务">
      <n-select v-model:value="taskId" :options="taskOptions" placeholder="选择任务" />
    </n-form-item>
    <n-form-item label="区间">
      <n-date-picker v-model:value="dateRange" type="daterange" clearable />
    </n-form-item>
    <n-form-item label="投入">
      <n-input-number v-model:value="percent" :min="0.01" :max="1" :step="0.05" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" attr-type="submit">分配</n-button>
    </n-form-item>
    <n-form-item v-if="error">
      <n-tag type="error">{{ error }}</n-tag>
    </n-form-item>
    <n-form-item v-if="impact">
      <n-text :type="impact.overloaded ? 'error' : 'success'">
        → 利用率 {{ Math.round(impact.utilization * 100) }}%{{ impact.overloaded ? " ⚠过载" : "" }}
      </n-text>
    </n-form-item>
  </n-form>
</template>
