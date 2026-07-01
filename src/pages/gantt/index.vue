<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useGanttProjectQuery, useGanttResourceQuery, useDependenciesForProjectQuery } from "@/services/api/gantt.api";
import { useUpdateAllocationMutation } from "@/services/api/allocations.api";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useProjectsStore } from "@/stores/projects";
import GanttTimeline from "@/components/GanttTimeline.vue";

const projects = useProjectsStore();
const projectsQuery = useListProjectsQuery();
const resourcesQuery = useListResourcesQuery();
const updateAllocation = useUpdateAllocationMutation();
const mode = ref<"project" | "resource">("project");
const focusId = ref<number | null>(null);
const err = ref<string | null>(null);
const start = ref("2026-06-29");
const end = ref("2026-08-09");
const resourceSelect = ref<number | null>(null);

const projectOptions = computed(() =>
  (projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: p.id })),
);
const resourceOptions = computed(() =>
  (resourcesQuery.data.value ?? []).map((r) => ({ label: r.name, value: r.id })),
);

const ganttProjectQuery = useGanttProjectQuery(computed(() => (mode.value === "project" ? focusId.value : null)));
const ganttResourceQuery = useGanttResourceQuery(computed(() => (mode.value === "resource" ? focusId.value : null)));
const depsQuery = useDependenciesForProjectQuery(computed(() => (mode.value === "project" ? focusId.value : null)));

const bars = computed(() => {
  if (mode.value === "resource") return ganttResourceQuery.data.value ?? [];
  return ganttProjectQuery.data.value ?? [];
});
const deps = computed(() => depsQuery.data.value ?? []);
const activeError = computed(() => ganttProjectQuery.error ?? ganttResourceQuery.error ?? depsQuery.error);

watch(
  () => activeError.value,
  (e) => {
    err.value = e ? (e instanceof Error ? e.message : String(e)) : null;
  },
);

watch(
  () => projects.current,
  (id) => {
    if (mode.value === "project" && id != null) {
      focusId.value = id;
    }
  },
  { immediate: true },
);

async function onProjectChange(value: unknown) {
  const id = Number(value);
  if (!Number.isNaN(id)) {
    projects.select(id);
    mode.value = "project";
    focusId.value = id;
  }
}

async function onResource(value: unknown) {
  const id = Number(value);
  if (Number.isNaN(id)) return;
  mode.value = "resource";
  focusId.value = id;
  resourceSelect.value = id;
}

function toProjectMode() {
  mode.value = "project";
  if (projects.current != null) focusId.value = projects.current;
}

async function onBarUpdate(id: number, start: string, end: string, percent: number) {
  try {
    await updateAllocation.mutateAsync({ id, start, end, percent });
  } catch (e: unknown) {
    err.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <h2 class="text-2xl font-bold mt-0">Gantt</h2>
  <div class="flex items-center gap-2 mb-2">
    <span>模式：</span>
    <Button :disabled="mode === 'project'" @click="toProjectMode">项目</Button>
    <Select
      :model-value="projects.current ?? undefined"
      :disabled="mode !== 'project'"
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
  <GanttTimeline :start="start" :end="end" :bars="bars" :deps="deps" @update="onBarUpdate" />
</template>
