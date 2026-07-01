<script setup lang="ts">
import { computed } from "vue";
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
import type { SuggestionItem } from "@/types";

const opt = useOptimizationStore();

const acceptedIds = computed(() =>
  opt.suggestions.filter((s) => s.status === "accepted").map((s) => s.id)
);

function kindLabel(s: SuggestionItem): string {
  const map: Record<string, string> = {
    swap_resource: "换人",
    change_percent: "改占比",
    widen_window: "放宽任务窗",
    drop_dependency: "解依赖",
    add_resource: "加资源",
    widen_resource_window: "放宽资源窗",
    change_resource_capacity: "改容量",
    upsert_resource_skill: "补技能",
  };
  return map[s.suggestion.kind] ?? s.suggestion.kind;
}

async function doRerun() {
  if (!opt.current) return;
  await opt.rerun(opt.current.run_id, acceptedIds.value);
}

async function pick(which: "current" | "parent") {
  const pickRun = which === "current" ? opt.current : opt.compareTarget;
  const other = which === "current" ? opt.compareTarget : opt.current;
  if (pickRun) await opt.accept(pickRun.run_id);
  if (other) await opt.reject(other.run_id);
  opt.compareTarget = null;
}
</script>

<template>
  <div v-if="opt.suggestions.length || opt.compareTarget" class="mt-6 space-y-6">
    <!-- 建议区 -->
    <div v-if="opt.suggestions.length">
      <h3 class="text-lg font-semibold mb-2">AI 改进建议</h3>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-10">采纳</TableHead>
            <TableHead>类型</TableHead>
            <TableHead>理由</TableHead>
            <TableHead>状态</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="s in opt.suggestions" :key="s.id">
            <TableCell>
              <input
                type="checkbox"
                :checked="s.status === 'accepted'"
                :disabled="s.status === 'applied'"
                @change="opt.toggleSuggestion(s.id, ($event.target as HTMLInputElement).checked)"
              />
            </TableCell>
            <TableCell>{{ kindLabel(s) }}</TableCell>
            <TableCell class="text-sm text-muted-foreground">{{ s.rationale_md }}</TableCell>
            <TableCell>{{ s.status }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <Button class="mt-2" :disabled="!acceptedIds.length || opt.busy" @click="doRerun">
        用选中建议重跑求解器
      </Button>
    </div>

    <!-- 对比区 -->
    <div v-if="opt.compareTarget && opt.current" class="grid grid-cols-2 gap-4">
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">原方案 #{{ opt.compareTarget.run_id }}</h4>
        <div class="text-sm">
          综合 {{ opt.compareTarget.plan.solution.metrics.overall.toFixed(0) }}
        </div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('parent')">
          采纳此方案
        </Button>
      </div>
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">重跑方案 #{{ opt.current.run_id }}</h4>
        <div class="text-sm">
          综合 {{ opt.current.plan.solution.metrics.overall.toFixed(0) }}
        </div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('current')">
          采纳此方案
        </Button>
      </div>
    </div>
  </div>
</template>
