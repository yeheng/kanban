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
import type { RunResult, SuggestionItem } from "@/types";

const props = defineProps<{
  run: RunResult | null;
  compareTarget: RunResult | null;
  suggestions: SuggestionItem[];
  busy?: boolean;
}>();

const emit = defineEmits<{
  (e: "rerun", suggestionIds: number[]): void;
  (e: "accept", runId: number): void;
  (e: "reject", runId: number): void;
  (e: "toggle", id: number, on: boolean): void;
}>();

const acceptedIds = computed(() =>
  props.suggestions.filter((s) => s.status === "accepted").map((s) => s.id)
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

function doRerun() {
  if (!acceptedIds.value.length) return;
  emit("rerun", acceptedIds.value);
}

function pick(which: "current" | "parent") {
  const pickRun = which === "current" ? props.run : props.compareTarget;
  const other = which === "current" ? props.compareTarget : props.run;
  if (pickRun) emit("accept", pickRun.run_id);
  if (other) emit("reject", other.run_id);
}
</script>

<template>
  <div v-if="suggestions.length || compareTarget" class="mt-6 space-y-6">
    <!-- 建议区 -->
    <div v-if="suggestions.length">
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
          <TableRow v-for="s in suggestions" :key="s.id">
            <TableCell>
              <input
                type="checkbox"
                :checked="s.status === 'accepted'"
                :disabled="s.status === 'applied' || busy"
                @change="emit('toggle', s.id, ($event.target as HTMLInputElement).checked)"
              />
            </TableCell>
            <TableCell>{{ kindLabel(s) }}</TableCell>
            <TableCell class="text-sm text-muted-foreground">{{ s.rationale_md }}</TableCell>
            <TableCell>{{ s.status }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <Button class="mt-2" :disabled="!acceptedIds.length || busy" @click="doRerun">
        用选中建议重跑求解器
      </Button>
    </div>

    <!-- 对比区 -->
    <div v-if="compareTarget && run" class="grid grid-cols-2 gap-4">
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">原方案 #{{ compareTarget.run_id }}</h4>
        <div class="text-sm">
          综合 {{ compareTarget.plan.solution.metrics.overall.toFixed(0) }}
        </div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('parent')">
          采纳此方案
        </Button>
      </div>
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">重跑方案 #{{ run.run_id }}</h4>
        <div class="text-sm">
          综合 {{ run.plan.solution.metrics.overall.toFixed(0) }}
        </div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('current')">
          采纳此方案
        </Button>
      </div>
    </div>
  </div>
</template>
