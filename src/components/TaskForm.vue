<script setup lang="ts">
import { computed, ref } from "vue";
import { NForm, NFormItem, NInput, NInputNumber, NSelect, NButton, NSwitch, NDatePicker } from "naive-ui";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";
import { fmtDateOrNull } from "../utils/date";

const tasks = useTasksStore();
const projects = useProjectsStore();
const catalog = useCatalogStore();
const title = ref("");
const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);
const isLongTerm = ref(false);
const segmentKind = ref<string | null>(null);
const parentTaskId = ref<number | null>(null);
const startMs = ref<number | null>(null);
const endMs = ref<number | null>(null);

const skillOptions = computed(() =>
  catalog.skills.map((s) => ({ label: s.name, value: s.id })),
);
const tagOptions = computed(() =>
  catalog.tags.map((t) => ({ label: t.name, value: t.id })),
);
// Parent candidates: long-term tasks in the current project (top-level, not segments).
const parentOptions = computed(() =>
  tasks.tasks
    .filter((t) => t.title)
    .map((t) => ({ label: t.title, value: t.id })),
);
const segmentKindOptions = [
  { label: "阶段 phase", value: "phase" },
  { label: "里程碑 milestone", value: "milestone" },
  { label: "分段 segment", value: "segment" },
];

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as [number, number, boolean, number]);
  await tasks.create({
    projectId: projects.current,
    title: title.value,
    estimatePd: estimate.value,
    start: fmtDateOrNull(startMs.value),
    end: fmtDateOrNull(endMs.value),
    skillReqs,
    tagIds: selectedTags.value,
    isLongTerm: isLongTerm.value,
    parentTaskId: segmentKind.value ? parentTaskId.value : null,
    segmentKind: segmentKind.value,
  });
  title.value = "";
  estimate.value = 1;
  selectedSkills.value = [];
  selectedTags.value = [];
  isLongTerm.value = false;
  segmentKind.value = null;
  parentTaskId.value = null;
  startMs.value = null;
  endMs.value = null;
}
</script>

<template>
  <n-form inline>
    <n-form-item label="标题">
      <n-input v-model:value="title" placeholder="任务标题" @keyup.enter="submit" />
    </n-form-item>
    <n-form-item label="PD">
      <n-input-number v-model:value="estimate" :min="0" />
    </n-form-item>
    <n-form-item label="起始日">
      <n-date-picker v-model:value="startMs" type="date" clearable />
    </n-form-item>
    <n-form-item label="截止日">
      <n-date-picker v-model:value="endMs" type="date" clearable />
    </n-form-item>
    <n-form-item label="技能">
      <n-select v-model:value="selectedSkills" multiple :options="skillOptions" placeholder="选择技能" />
    </n-form-item>
    <n-form-item label="标签">
      <n-select v-model:value="selectedTags" multiple :options="tagOptions" placeholder="选择标签" />
    </n-form-item>
    <n-form-item label="长期任务">
      <n-switch v-model:value="isLongTerm" />
    </n-form-item>
    <n-form-item label="分段类型">
      <n-select
        v-model:value="segmentKind"
        :options="segmentKindOptions"
        clearable
        placeholder="无"
      />
    </n-form-item>
    <n-form-item v-if="segmentKind" label="父任务">
      <n-select
        v-model:value="parentTaskId"
        :options="parentOptions"
        placeholder="选择父任务"
      />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="submit">新建任务</n-button>
    </n-form-item>
  </n-form>
</template>
