<script setup lang="ts">
import { ref } from "vue";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
const name = ref(""); const priority = ref(5); const budget = ref(0);
async function submit() { if (!name.value.trim()) return; await projects.create(name.value, priority.value, budget.value); name.value = ""; }
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="name" placeholder="项目名" />
    <input v-model.number="priority" type="number" min="1" max="9" />
    <input v-model.number="budget" type="number" min="0" placeholder="budget PD" />
    <button>新建项目</button>
  </form>
</template>