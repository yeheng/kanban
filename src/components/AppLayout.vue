<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NLayout, NLayoutSider, NLayoutContent } from "naive-ui";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const ready = ref(false);

onMounted(async () => {
  for (let i = 0; i < 40; i++) {
    try { await projects.load(); await catalog.load(); ready.value = true; return; }
    catch { await new Promise((r) => setTimeout(r, 100)); }
  }
});
</script>

<template>
  <n-layout has-sider style="height: 100vh">
    <n-layout-sider bordered content-style="padding:16px" :width="200">
      <h3 style="margin-top:0">HR Kanban</h3>
      <router-link to="/kanban" style="display:block;padding:6px 0">看板 Kanban</router-link>
      <router-link to="/projects" style="display:block;padding:6px 0">项目 Projects</router-link>
      <router-link to="/resources" style="display:block;padding:6px 0">资源 Resources</router-link>
      <router-link to="/dashboard" style="display:block;padding:6px 0">仪表盘 Dashboard</router-link>
      <router-link to="/allocations" style="display:block;padding:6px 0">分配 Allocations</router-link>
      <router-link to="/calendar" style="display:block;padding:6px 0">日历 Calendar</router-link>
      <router-link to="/gantt" style="display:block;padding:6px 0">甘特图 Gantt</router-link>
      <router-link to="/calendar-grid" style="display:block;padding:6px 0">占用网格 Calendar Grid</router-link>
      <router-link to="/ai" style="display:block;padding:6px 0">AI 优化 Optimization</router-link>
      <router-link to="/reports" style="display:block;padding:6px 0">报表 Reports</router-link>
      <hr />
      <small>项目：</small>
      <select v-model.number="projects.current" @change="projects.select(projects.current!)">
        <option v-for="p in projects.items" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </n-layout-sider>
    <n-layout-content content-style="padding:16px">
      <div v-if="!ready">正在打开数据库…</div>
      <router-view v-else />
    </n-layout-content>
  </n-layout>
</template>