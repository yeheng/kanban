<script setup lang="ts">
import { ref } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";
import { api } from "../api";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const title = ref(""); const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as [number, number, boolean, number]);
  await api.createTask({
    projectId: projects.current, title: title.value, estimatePd: estimate.value,
    start: null, end: null, skillReqs, tagIds: selectedTags.value,
  });
  title.value = ""; estimate.value = 1; selectedSkills.value = []; selectedTags.value = [];
  await useTasksStore().load(projects.current);
}
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="title" placeholder="任务标题" />
    <input v-model.number="estimate" type="number" min="0" placeholder="PD" />
    <select v-model="selectedSkills" multiple>
      <option v-for="s in catalog.skills" :key="s.id" :value="s.id">{{ s.name }}</option>
    </select>
    <select v-model="selectedTags" multiple>
      <option v-for="t in catalog.tags" :key="t.id" :value="t.id">{{ t.name }}</option>
    </select>
    <button>新建任务</button>
  </form>
</template>