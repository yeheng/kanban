<script setup lang="ts">
import { computed, ref } from "vue";
import { NForm, NFormItem, NInput, NInputNumber, NSelect, NButton } from "naive-ui";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";
import { api } from "../api";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const title = ref("");
const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);

const skillOptions = computed(() =>
  catalog.skills.map((s) => ({ label: s.name, value: s.id })),
);
const tagOptions = computed(() =>
  catalog.tags.map((t) => ({ label: t.name, value: t.id })),
);

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as [number, number, boolean, number]);
  await api.createTask({
    projectId: projects.current, title: title.value, estimatePd: estimate.value,
    start: null, end: null, skillReqs, tagIds: selectedTags.value,
  });
  title.value = "";
  estimate.value = 1;
  selectedSkills.value = [];
  selectedTags.value = [];
  await useTasksStore().load(projects.current);
}
</script>

<template>
  <n-form inline @submit.prevent="submit">
    <n-form-item label="标题">
      <n-input v-model:value="title" placeholder="任务标题" />
    </n-form-item>
    <n-form-item label="PD">
      <n-input-number v-model:value="estimate" :min="0" />
    </n-form-item>
    <n-form-item label="技能">
      <n-select v-model:value="selectedSkills" multiple :options="skillOptions" placeholder="选择技能" />
    </n-form-item>
    <n-form-item label="标签">
      <n-select v-model:value="selectedTags" multiple :options="tagOptions" placeholder="选择标签" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" attr-type="submit">新建任务</n-button>
    </n-form-item>
  </n-form>
</template>
