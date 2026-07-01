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
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import {
  useRunOptimizationMutation,
  useListOptimizationRunsQuery,
  useGetOptimizationRunQuery,
  useApplySolutionMutation,
  useRejectSolutionMutation,
  useListSuggestionsQuery,
  useRerunMutation,
  useSetSuggestionStatusMutation,
} from "@/services/api/optimization.api";
import { useProjectsStore } from "@/stores/projects";
import WeightsPanel from "@/components/WeightsPanel.vue";
import PlanReview from "@/components/PlanReview.vue";
import PlanCompare from "@/components/PlanCompare.vue";
import type { ObjectiveWeights, RunResult } from "@/types";

const projects = useProjectsStore();
const page = ref(1);
const pageSize = ref(10);
const offset = computed(() => (page.value - 1) * pageSize.value);
const runsQuery = useListOptimizationRunsQuery(offset, pageSize);
const runOptimization = useRunOptimizationMutation();
const applySolution = useApplySolutionMutation();
const rejectSolution = useRejectSolutionMutation();
const rerun = useRerunMutation();
const setSuggestionStatus = useSetSuggestionStatusMutation();

const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
const currentRun = ref<RunResult | null>(null);
const compareTarget = ref<RunResult | null>(null);
const viewingRunId = ref<number | null>(null);

const viewRunQuery = useGetOptimizationRunQuery(viewingRunId);
const suggestionsQuery = useListSuggestionsQuery(computed(() => currentRun.value?.run_id ?? null));
const viewError = ref<string | null>(null);

const isRunning = computed(() => runOptimization.isPending.value);

watch(() => viewRunQuery.data.value, (run) => {
  if (run) {
    currentRun.value = run;
    viewError.value = null;
  }
});

watch(() => viewRunQuery.error.value, (e) => {
  if (e) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
});

const totalPages = computed(() =>
  Math.max(1, Math.ceil((runsQuery.data.value?.total ?? 0) / pageSize.value)),
);

const displayedRows = computed(() => runsQuery.data.value?.rows ?? []);

const busy = computed(() =>
  runOptimization.isPending.value ||
  rerun.isPending.value ||
  applySolution.isPending.value ||
  rejectSolution.isPending.value ||
  setSuggestionStatus.isPending.value,
);

async function runForCurrent() {
  if (projects.current == null || busy.value) return;
  viewError.value = null;
  compareTarget.value = null;
  try {
    const result = await runOptimization.mutateAsync({ projectId: projects.current, weights: weights.value });
    currentRun.value = result;
    viewingRunId.value = null;
  } catch (e: unknown) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
}

async function loadRun(id: number) {
  if (busy.value) return;
  viewError.value = null;
  currentRun.value = null;
  compareTarget.value = null;
  viewingRunId.value = id;
}

async function doRerun(suggestionIds: number[]) {
  if (!currentRun.value || busy.value) return;
  viewError.value = null;
  const parent = currentRun.value;
  try {
    const result = await rerun.mutateAsync({ runId: parent.run_id, suggestionIds });
    currentRun.value = result;
    compareTarget.value = parent;
  } catch (e: unknown) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
}

async function toggleSuggestion(id: number, on: boolean) {
  try {
    await setSuggestionStatus.mutateAsync({ id, status: on ? "accepted" : "skipped" });
  } catch (e: unknown) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
}

async function accept(runId: number) {
  try {
    await applySolution.mutateAsync(runId);
    currentRun.value = null;
    compareTarget.value = null;
    viewingRunId.value = null;
  } catch (e: unknown) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
}

async function reject(runId: number) {
  try {
    await rejectSolution.mutateAsync(runId);
    currentRun.value = null;
    compareTarget.value = null;
    viewingRunId.value = null;
  } catch (e: unknown) {
    viewError.value = e instanceof Error ? e.message : String(e);
  }
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
        :disabled="projects.current == null || busy"
        @click="runForCurrent"
      >
        {{ isRunning ? "求解中…" : "为当前项目运行优化" }}
      </Button>
    </div>
    <div class="flex-1">
      <PlanReview v-if="currentRun" :run="currentRun" @accept="accept" @reject="reject" />
      <div v-else-if="viewRunQuery.isPending.value" class="space-y-2">
        <Skeleton class="h-6 w-48" />
        <Skeleton class="h-32 w-full" />
      </div>
      <Alert v-else-if="viewError" variant="destructive">
        <AlertDescription>{{ viewError }}</AlertDescription>
      </Alert>
      <span v-else class="text-muted-foreground">运行优化后在此查看建议方案。</span>
      <PlanCompare
        :run="currentRun"
        :compare-target="compareTarget"
        :suggestions="suggestionsQuery.data.value ?? []"
        :busy="busy"
        @rerun="doRerun"
        @accept="accept"
        @reject="reject"
        @toggle="toggleSuggestion"
      />
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
        <TableHead>操作</TableHead>
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
        <TableCell>
          <Button variant="outline" size="sm" @click="loadRun(row.id)">
            查看方案
          </Button>
        </TableCell>
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
    <span>共 {{ runsQuery.data.value?.total ?? 0 }} 条</span>
  </div>
</template>
