<script setup lang="ts">
import { onMounted } from "vue";
import { useOptimizationStore } from "../stores/optimization";
import { useProjectsStore } from "../stores/projects";
import WeightsPanel from "../components/WeightsPanel.vue";
import PlanReview from "../components/PlanReview.vue";

const opt = useOptimizationStore();
const projects = useProjectsStore();
onMounted(() => opt.loadHistory());
function runForCurrent() {
  if (projects.current != null) opt.run(projects.current);
}
</script>
<template>
  <h2 style="margin-top:0">AI 优化 / Optimization</h2>
  <div style="display:flex;gap:24px;align-items:flex-start">
    <div>
      <h3>目标权重</h3>
      <WeightsPanel />
      <button :disabled="projects.current == null || opt.busy" @click="runForCurrent">
        {{ opt.busy ? "求解中…" : "为当前项目运行优化" }}
      </button>
    </div>
    <div style="flex:1">
      <PlanReview v-if="opt.current" />
      <p v-else style="color:#888">运行优化后在此查看建议方案。</p>
    </div>
  </div>

  <h3>历史运行</h3>
  <table border="1" cellpadding="4" style="border-collapse:collapse">
    <tr><th>#</th><th>状态</th><th>评分</th><th>已采纳</th><th>时间</th></tr>
    <tr v-for="r in opt.history" :key="r.id">
      <td>{{ r.id }}</td><td>{{ r.status }}</td>
      <td>{{ r.score_overall != null ? r.score_overall.toFixed(0) : "-" }}</td>
      <td>{{ r.applied ? "是" : "否" }}</td><td>{{ r.created_at }}</td>
    </tr>
  </table>
</template>
