<script setup lang="ts">
import { NH2, NH3, NList, NListItem, NThing, NTag, NDivider, NText, NPopconfirm, NButton, NSpace } from "naive-ui";
import ProjectForm from "../components/ProjectForm.vue";
import TaskForm from "../components/TaskForm.vue";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
</script>

<template>
  <n-h2>项目 / Projects</n-h2>
  <ProjectForm />
  <n-list bordered hoverable>
    <n-list-item v-for="p in projects.items" :key="p.id">
      <n-thing>
        <template #header>
          <n-text :strong="p.id === projects.current" @click="projects.select(p.id)" style="cursor: pointer">
            {{ p.name }}
          </n-text>
        </template>
        <template #description>
          <n-space :size="4">
            <n-tag size="small" :bordered="false">优先级 {{ p.priority }}</n-tag>
            <n-tag size="small" :bordered="false" type="info">预算 {{ p.budget_pd }} PD</n-tag>
          </n-space>
        </template>
        <template #suffix>
          <n-popconfirm @positive-click="projects.remove(p.id)">
            <template #trigger>
              <n-button size="small" type="error" quaternary>删除</n-button>
            </template>
            确定删除项目 "{{ p.name }}" 吗？
          </n-popconfirm>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
  <n-divider />
  <n-h3>在当前项目新建任务</n-h3>
  <TaskForm v-if="projects.current" />
  <n-text v-else depth="3">请先选择一个项目。</n-text>
</template>
