<script setup lang="ts">
import ProjectForm from "../components/ProjectForm.vue";
import TaskForm from "../components/TaskForm.vue";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
</script>
<template>
  <h2>项目 / Projects</h2>
  <ProjectForm />
  <ul>
    <li v-for="p in projects.items" :key="p.id" :style="p.id === projects.current ? 'font-weight:bold' : ''">
      <a href="#" @click.prevent="projects.select(p.id)">{{ p.name }}</a> — 优先级 {{ p.priority }} · 预算 {{ p.budget_pd }} PD
    </li>
  </ul>
  <hr />
  <h3>在当前项目新建任务</h3>
  <TaskForm v-if="projects.current" />
  <p v-else>请先选择一个项目。</p>
</template>