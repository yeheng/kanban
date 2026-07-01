<script setup lang="ts">
import { computed } from "vue";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useOptimizationStore } from "@/stores/optimization";
import type { ScoredAssignment } from "@/types";

const opt = useOptimizationStore();
function pct(v: number) { return Math.round(v) + "%"; }

interface StatItem { label: string; value: string; }
const stats = computed<StatItem[]>(() => [
  { label: "综合评分", value: pct(opt.current!.plan.solution.metrics.overall) },
  { label: "技能", value: pct(opt.current!.plan.solution.metrics.skill_fit) },
  { label: "排期覆盖", value: pct(opt.current!.plan.solution.metrics.scheduled_ratio) },
]);

function assignmentKey(a: ScoredAssignment, i: number) {
  return `${a.resource_id}-${a.task_id}-${a.start}-${a.end}-${i}`;
}
</script>

<template>
  <div v-if="opt.current" class="space-y-4">
    <h3 class="text-2xl font-semibold tracking-tight">
      方案 #{{ opt.current.run_id }}
    </h3>

    <div class="flex flex-wrap gap-4">
      <Card v-for="stat in stats" :key="stat.label" class="min-w-[120px]">
        <CardHeader class="pb-2">
          <div class="text-sm text-muted-foreground">{{ stat.label }}</div>
        </CardHeader>
        <CardContent>
          <div class="text-2xl font-bold">{{ stat.value }}</div>
        </CardContent>
      </Card>
    </div>

    <h4 class="text-lg font-semibold tracking-tight">
      已分配 ({{ opt.current.plan.solution.assignments.length }})
    </h4>
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="w-20">资源</TableHead>
          <TableHead class="w-20">任务</TableHead>
          <TableHead>区间</TableHead>
          <TableHead class="w-20">投入</TableHead>
          <TableHead class="w-20">匹配分</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow
          v-for="(a, i) in opt.current.plan.solution.assignments"
          :key="assignmentKey(a, i)"
        >
          <TableCell>
            <div class="font-medium">{{ a.resource_name || `资源 #${a.resource_id}` }}</div>
            <div v-if="a.resource_name" class="text-muted-foreground text-xs">#{{ a.resource_id }}</div>
          </TableCell>
          <TableCell>
            <div class="font-medium">{{ a.task_title || `任务 #${a.task_id}` }}</div>
            <div v-if="a.task_title" class="text-muted-foreground text-xs">#{{ a.task_id }}</div>
          </TableCell>
          <TableCell>{{ a.start }} → {{ a.end }}</TableCell>
          <TableCell>{{ Math.round(a.percent * 100) }}%</TableCell>
          <TableCell>{{ Math.round(a.score * 100) }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>

    <Alert
      v-if="opt.current.plan.solution.unscheduled.length"
      variant="default"
      class="border-yellow-200 bg-yellow-50 text-yellow-800"
    >
      <AlertDescription class="text-yellow-800">
        ⚠ 未排期任务：{{ opt.current.plan.solution.unscheduled.join(", ") }}
      </AlertDescription>
    </Alert>

    <h4 class="text-lg font-semibold tracking-tight">解释</h4>
    <span class="text-muted-foreground plan-review__explanation block">
      {{ opt.current.plan.explanation_md }}
    </span>

    <div class="flex flex-wrap gap-2">
      <Button @click="opt.accept(opt.current!.run_id)">✓ 采纳（写入分配）</Button>
      <Button variant="destructive" @click="opt.reject(opt.current!.run_id)">
        ✗ 拒绝
      </Button>
    </div>
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
