<script setup lang="ts">
import { computed, h, onMounted, ref } from "vue";
import { NLayout, NLayoutSider, NLayoutContent, NMenu, NSelect, NSpin, NDivider, NText } from "naive-ui";
import type { MenuOption } from "naive-ui";
import { RouterLink, useRoute } from "vue-router";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";
import { useUnitStore } from "../stores/unit";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const unit = useUnitStore();
const ready = ref(false);
const route = useRoute();

const menuOptions = computed<MenuOption[]>(() => [
  { label: () => h(RouterLink, { to: "/kanban" }, { default: () => "看板 Kanban" }), key: "kanban" },
  { label: () => h(RouterLink, { to: "/projects" }, { default: () => "项目 Projects" }), key: "projects" },
  { label: () => h(RouterLink, { to: "/resources" }, { default: () => "资源 Resources" }), key: "resources" },
  { label: () => h(RouterLink, { to: "/catalog" }, { default: () => "技能标签 Catalog" }), key: "catalog" },
  { label: () => h(RouterLink, { to: "/teams" }, { default: () => "团队 Teams" }), key: "teams" },
  { label: () => h(RouterLink, { to: "/dashboard" }, { default: () => "仪表盘 Dashboard" }), key: "dashboard" },
  { label: () => h(RouterLink, { to: "/allocations" }, { default: () => "分配 Allocations" }), key: "allocations" },
  { label: () => h(RouterLink, { to: "/calendar" }, { default: () => "日历 Calendar" }), key: "calendar" },
  { label: () => h(RouterLink, { to: "/gantt" }, { default: () => "甘特图 Gantt" }), key: "gantt" },
  { label: () => h(RouterLink, { to: "/calendar-grid" }, { default: () => "占用网格 Calendar Grid" }), key: "calendar-grid" },
  { label: () => h(RouterLink, { to: "/ai" }, { default: () => "AI 优化 Optimization" }), key: "ai" },
  { label: () => h(RouterLink, { to: "/reports" }, { default: () => "报表 Reports" }), key: "reports" },
]);

const activeKey = computed(() => route.path.replace(/^\//, ""));

const projectOptions = computed(() =>
  projects.items.map((p) => ({ label: p.name, value: p.id })),
);

function onProjectChange(id: number | null) {
  if (id != null) projects.select(id);
}

onMounted(async () => {
  for (let i = 0; i < 40; i++) {
    try { await projects.load(); await catalog.load(); ready.value = true; return; }
    catch { await new Promise((r) => setTimeout(r, 100)); }
  }
});
</script>

<template>
  <n-layout has-sider style="height: 100vh">
    <n-layout-sider bordered content-style="padding: 16px" :width="200">
      <n-text strong style="font-size: 16px">HR Kanban</n-text>
      <n-menu :options="menuOptions" :value="activeKey" />
      <n-divider />
      <n-text depth="3" style="font-size: 12px">项目</n-text>
      <n-select
        :value="projects.current"
        :options="projectOptions"
        placeholder="选择项目"
        size="small"
        @update:value="onProjectChange"
      />
      <n-divider />
      <n-text depth="3" style="font-size: 12px">单位</n-text>
      <n-select v-model:value="unit.unit" :options="unit.options" size="small" />
    </n-layout-sider>
    <n-layout-content content-style="padding: 16px">
      <n-spin v-if="!ready" size="large">
        <template #description>正在打开数据库…</template>
      </n-spin>
      <router-view v-else />
    </n-layout-content>
  </n-layout>
</template>
