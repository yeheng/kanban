<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  useRunOptimizationMutation,
  useListOptimizationRunsQuery,
  useGetOptimizationRunQuery,
  useApplySolutionMutation,
  useRejectSolutionMutation,
} from "@/services/api/optimization.api";
import { useProjectsStore } from "@/stores/projects";
import WeightsPanel from "@/components/WeightsPanel.vue";
import PlanReview from "@/components/PlanReview.vue";
import type { ObjectiveWeights, RunResult } from "@/types";

const projects = useProjectsStore();
const runsQuery = useListOptimizationRunsQuery(null);
const runOptimization = useRunOptimizationMutation();
const applySolution = useApplySolutionMutation();
const rejectSolution = useRejectSolutionMutation();

const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
const currentRun = ref<RunResult | null>(null);
const viewingRunId = ref<number | null>(null);
const page = ref(1);
const pageSize = ref(10);

const viewRunQuery = useGetOptimizationRunQuery(viewingRunId);

watch(() => viewRunQuery.data.value, (run) => {
  if (run) currentRun.value = run;
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil((runsQuery.data.value?.length ?? 0) / pageSize.value)),
);

const displayedRows = computed(() => {
  const rows = runsQuery.data.value ?? [];
  const start = (page.value - 1) * pageSize.value;
  return rows.slice(start, start + pageSize.value);
});

async function runForCurrent() {
  if (projects.current == null) return;
  const result = await runOptimization.mutateAsync({ projectId: projects.current, weights: weights.value });
  currentRun.value = result;
  viewingRunId.value = null;
}

async function loadRun(id: number) {
  viewingRunId.value = id;
  // The watch on viewRunQuery.data will set currentRun when data arrives.
}

async function accept(runId: number) {
  await applySolution.mutateAsync(runId);
  currentRun.value = null;
  viewingRunId.value = null;
}

async function reject(runId: number) {
  await rejectSolution.mutateAsync(runId);
  currentRun.value = null;
  viewingRunId.value = null;
}

function setPage(n: number) {
  page.value = Math.max(1, Math.min(n, totalPages.value));
}

function setPageSize(n: number) {
  pageSize.value = Math.max(1, n);
  page.value = 1;
}
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight">AI 优化 / Optimization</h2>
  <div class="flex items-start gap-6">
    <div>
      <h3 class="text-xl font-bold tracking-tight">目标权重</h3>
      <WeightsPanel v-model="weights" />
      <Button
        :disabled="projects.current == null || runOptimization.isPending"
        @click="runForCurrent"
      >
        {{ runOptimization.isPending ? "求解中…" : "为当前项目运行优化" }}
      </Button>
    </div>
    <div class="flex-1">
      <PlanReview v-if="currentRun" :run="currentRun" @accept="accept" @reject="reject" />
      <span v-else class="text-muted-foreground">运行优化后在此查看建议方案。</span>
    </div>
  </div>

  <h3 class="text-xl font-bold tracking-tight">历史运行</h3>
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>#</TableHead>
        <TableHead>状态</TableHead>
        <TableHead>评分</TableHead>
        <TableHead>已采纳</TableHead>
        <TableHead>时间</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow v-for="row in displayedRows" :key="row.id" @click="loadRun(row.id)">
        <TableCell>{{ row.id }}</TableCell>
        <TableCell>{{ row.status }}</TableCell>
        <TableCell>
          {{ row.score_overall != null ? row.score_overall.toFixed(0) : "-" }}
        </TableCell>
        <TableCell>{{ row.applied ? "是" : "否" }}</TableCell>
        <TableCell>{{ row.created_at }}</TableCell>
      </TableRow>
    </TableBody>
  </Table>
  <div class="flex items-center gap-2 text-sm">
    <Button variant="outline" :disabled="page <= 1" @click="setPage(page - 1)">上一页</Button>
    <span>第 {{ page }} / {{ totalPages }} 页</span>
    <Button variant="outline" :disabled="page >= totalPages" @click="setPage(page + 1)">下一页</Button>
    <select :value="pageSize" @change="(e) => setPageSize(Number((e.target as HTMLSelectElement).value))">
      <option :value="10">10</option>
      <option :value="20">20</option>
      <option :value="50">50</option>
    </select>
    <span>共 {{ runsQuery.data.value?.length ?? 0 }} 条</span>
  </div>
</template>
