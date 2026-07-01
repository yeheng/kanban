<script setup lang="ts">
import { computed, onMounted } from "vue";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useOptimizationStore } from "@/stores/optimization";
import { useProjectsStore } from "@/stores/projects";
import WeightsPanel from "@/components/WeightsPanel.vue";
import PlanReview from "@/components/PlanReview.vue";

const opt = useOptimizationStore();
const projects = useProjectsStore();
onMounted(() => opt.loadHistory());
function runForCurrent() {
  if (projects.current != null) opt.run(projects.current);
}
const totalPages = computed(() =>
  Math.max(1, Math.ceil(opt.history.total / opt.pageSize))
);
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight">AI 优化 / Optimization</h2>
  <div class="flex items-start gap-6">
    <div>
      <h3 class="text-xl font-bold tracking-tight">目标权重</h3>
      <WeightsPanel />
      <Button
        :disabled="projects.current == null || opt.busy"
        @click="runForCurrent"
      >
        {{ opt.busy ? "求解中…" : "为当前项目运行优化" }}
      </Button>
    </div>
    <div class="flex-1">
      <PlanReview v-if="opt.current" />
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
        <TableHead>操作</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableRow v-for="row in opt.history.rows" :key="row.id">
        <TableCell>{{ row.id }}</TableCell>
        <TableCell>{{ row.status }}</TableCell>
        <TableCell>
          {{ row.score_overall != null ? row.score_overall.toFixed(0) : "-" }}
        </TableCell>
        <TableCell>{{ row.applied ? "是" : "否" }}</TableCell>
        <TableCell>{{ row.created_at }}</TableCell>
        <TableCell>
          <Button variant="outline" size="sm" @click="opt.loadRun(row.id)">
            查看方案
          </Button>
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>

  <div class="flex items-center justify-between mt-4">
    <div class="text-sm text-muted-foreground">
      共 {{ opt.history.total }} 条 · 第 {{ opt.page }} / {{ totalPages }} 页
    </div>
    <div class="flex items-center gap-2">
      <select
        v-model="opt.pageSize"
        class="h-9 rounded-md border border-input bg-background px-2 text-sm"
        @change="opt.setPageSize(Number(opt.pageSize))"
      >
        <option :value="10">10 条/页</option>
        <option :value="20">20 条/页</option>
        <option :value="50">50 条/页</option>
      </select>
      <Button
        variant="outline"
        size="sm"
        :disabled="opt.page <= 1"
        @click="opt.setPage(opt.page - 1)"
      >
        上一页
      </Button>
      <Button
        variant="outline"
        size="sm"
        :disabled="opt.page >= totalPages"
        @click="opt.setPage(opt.page + 1)"
      >
        下一页
      </Button>
    </div>
  </div>
</template>
