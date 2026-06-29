<script setup lang="ts">
import { computed, h, onMounted } from "vue";
import { NH2, NH3, NSpace, NButton, NText, NDataTable } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useOptimizationStore } from "../stores/optimization";
import { useProjectsStore } from "../stores/projects";
import WeightsPanel from "../components/WeightsPanel.vue";
import PlanReview from "../components/PlanReview.vue";
import type { RunRow } from "../types";

const opt = useOptimizationStore();
const projects = useProjectsStore();
onMounted(() => opt.loadHistory());
function runForCurrent() {
  if (projects.current != null) opt.run(projects.current);
}

const historyColumns = computed<DataTableColumns<RunRow>>(() => [
  { title: "#", key: "id", width: 60 },
  { title: "状态", key: "status", width: 100 },
  { title: "评分", key: "score", render: (r) => r.score_overall != null ? r.score_overall.toFixed(0) : "-", width: 80 },
  { title: "已采纳", key: "applied", render: (r) => r.applied ? "是" : "否", width: 80 },
  { title: "时间", key: "created_at" },
]);
</script>

<template>
  <n-h2>AI 优化 / Optimization</n-h2>
  <n-space :size="24" align="start">
    <div>
      <n-h3>目标权重</n-h3>
      <WeightsPanel />
      <n-button
        type="primary"
        :disabled="projects.current == null || opt.busy"
        :loading="opt.busy"
        @click="runForCurrent"
      >
        {{ opt.busy ? "求解中…" : "为当前项目运行优化" }}
      </n-button>
    </div>
    <div style="flex: 1">
      <PlanReview v-if="opt.current" />
      <n-text v-else depth="3">运行优化后在此查看建议方案。</n-text>
    </div>
  </n-space>

  <n-h3>历史运行</n-h3>
  <n-data-table :columns="historyColumns" :data="opt.history" :bordered="true" />
</template>
