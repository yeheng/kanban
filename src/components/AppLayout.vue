<script setup lang="ts">
import { computed, h, onMounted, ref } from "vue";
import { NLayout, NLayoutSider, NLayoutContent, NMenu, NSelect, NSpin, NDivider, NText } from "naive-ui";
import type { MenuOption } from "naive-ui";
import { RouterLink, useRoute } from "vue-router";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const ready = ref(false);
const route = useRoute();

const renderLink = (to: string) => () => h(RouterLink, { to });

const menuOptions = computed<MenuOption[]>(() => [
  { label: renderLink("/kanban"), key: "kanban" },
  { label: renderLink("/projects"), key: "projects" },
  { label: renderLink("/resources"), key: "resources" },
  { label: renderLink("/dashboard"), key: "dashboard" },
  { label: renderLink("/allocations"), key: "allocations" },
  { label: renderLink("/calendar"), key: "calendar" },
  { label: renderLink("/gantt"), key: "gantt" },
  { label: renderLink("/calendar-grid"), key: "calendar-grid" },
  { label: renderLink("/ai"), key: "ai" },
  { label: renderLink("/reports"), key: "reports" },
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
    </n-layout-sider>
    <n-layout-content content-style="padding: 16px">
      <n-spin v-if="!ready" size="large">
        <template #description>正在打开数据库…</template>
      </n-spin>
      <router-view v-else />
    </n-layout-content>
  </n-layout>
</template>
