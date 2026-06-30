<script setup lang="ts">
import { computed, h } from "vue";
import { NH3, NH4, NDataTable, NAlert, NButton, NSpace, NText, NStatistic } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useOptimizationStore } from "../stores/optimization";
import type { ScoredAssignment } from "../types";

const opt = useOptimizationStore();
function pct(v: number) { return Math.round(v) + "%"; }

const assignmentColumns = computed<DataTableColumns<ScoredAssignment>>(() => [
  { title: "资源", key: "resource_id", render: (a) => `#${a.resource_id}`, width: 80 },
  { title: "任务", key: "task_id", render: (a) => `#${a.task_id}`, width: 80 },
  { title: "区间", key: "range", render: (a) => `${a.start} → ${a.end}` },
  { title: "投入", key: "percent", render: (a) => `${Math.round(a.percent * 100)}%`, width: 80 },
  { title: "匹配分", key: "score", render: (a) => `${Math.round(a.score * 100)}`, width: 80 },
]);
</script>

<template>
  <div v-if="opt.current">
    <n-h3>方案 #{{ opt.current.run_id }}</n-h3>
    <n-space :size="24">
      <n-statistic label="综合评分" :value="pct(opt.current.plan.solution.metrics.overall)" />
      <n-statistic label="技能" :value="pct(opt.current.plan.solution.metrics.skill_fit)" />
      <n-statistic label="排期覆盖" :value="pct(opt.current.plan.solution.metrics.scheduled_ratio)" />
    </n-space>

    <n-h4>已分配 ({{ opt.current.plan.solution.assignments.length }})</n-h4>
    <n-data-table
      :columns="assignmentColumns"
      :data="opt.current.plan.solution.assignments"
      :bordered="true"
      size="small"
    />

    <n-alert
      v-if="opt.current.plan.solution.unscheduled.length"
      type="warning"
      show-icon
      style="margin-top: 8px"
    >
      ⚠ 未排期任务：{{ opt.current.plan.solution.unscheduled.join(", ") }}
    </n-alert>

    <n-h4>解释</n-h4>
    <n-text depth="3" class="plan-review__explanation">
      {{ opt.current.plan.explanation_md }}
    </n-text>

    <n-space style="margin-top: 12px" :size="8">
      <n-button type="success" @click="opt.accept(opt.current!.run_id)">✓ 采纳（写入分配）</n-button>
      <n-button type="error" @click="opt.reject(opt.current!.run_id)">✗ 拒绝</n-button>
    </n-space>
  </div>
</template>

<style scoped>
.plan-review__explanation {
  display: block;
  white-space: pre-wrap;
  background: #f7f7fa;
  padding: 8px;
  border-radius: 4px;
  font-size: 13px;
}
</style>
