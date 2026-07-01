<script setup lang="ts">
import { computed, ref } from "vue";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Badge } from "@/components/ui/badge";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useCreateAllocationMutation } from "@/services/api/allocations.api";
import { useListResourcesQuery } from "@/services/api/resources.api";
import { useListTasksQuery } from "@/services/api/tasks.api";
import { useResourceSummaryQuery } from "@/services/api/workload.api";
import { useProjectsStore } from "@/stores/projects";
import { fmtDate, parseDateStrict } from "@/utils/date";

const createAllocation = useCreateAllocationMutation();
const resourcesQuery = useListResourcesQuery();
const projects = useProjectsStore();
const tasksQuery = useListTasksQuery(computed(() => projects.current));

const resourceId = ref<number | null>(null);
const taskId = ref<number | null>(null);
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-03")]);
const percent = ref(0.5);
const impact = ref<{ utilization: number; overloaded: boolean } | null>(null);
const error = ref<string | null>(null);

const resourceOptions = computed(() =>
  (resourcesQuery.data.value ?? []).map((r) => ({ label: r.name, value: r.id })),
);
const taskOptions = computed(() =>
  (tasksQuery.data.value ?? []).map((t) => ({ label: t.title, value: t.id })),
);

const resourceIdSelect = computed<string | number | undefined>({
  get: () => resourceId.value ?? undefined,
  set: (v) => { resourceId.value = v == null ? null : Number(v); },
});

const taskIdSelect = computed<string | number | undefined>({
  get: () => taskId.value ?? undefined,
  set: (v) => { taskId.value = v == null ? null : Number(v); },
});

const startStr = computed(() => fmtDate(dateRange.value[0]));
const endStr = computed(() => fmtDate(dateRange.value[1]));
const resourceSummaryQuery = useResourceSummaryQuery(resourceId, startStr, endStr);

async function submit() {
  error.value = null;
  if (resourceId.value == null || taskId.value == null || projects.current == null) return;
  const start = startStr.value;
  const end = endStr.value;
  try {
    await createAllocation.mutateAsync({
      resourceId: resourceId.value,
      taskId: taskId.value,
      start,
      end,
      percent: percent.value,
      projectId: projects.current,
    });
    const s = resourceSummaryQuery.data.value;
    impact.value = s ? { utilization: s.utilization, overloaded: s.overloaded } : null;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label>资源</Label>
      <Select v-model="resourceIdSelect">
        <SelectTrigger>
          <SelectValue placeholder="选择资源" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem
            v-for="option in resourceOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label>任务</Label>
      <Select v-model="taskIdSelect">
        <SelectTrigger>
          <SelectValue placeholder="选择任务" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem
            v-for="option in taskOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="grid gap-2">
      <Label>区间</Label>
      <DateRangePicker v-model="dateRange" />
    </div>

    <div class="grid gap-2">
      <Label>投入</Label>
      <NumberField v-model="percent" :min="0.01" :max="1" :step="0.05">
        <NumberFieldContent>
          <NumberFieldDecrement />
          <NumberFieldInput />
          <NumberFieldIncrement />
        </NumberFieldContent>
      </NumberField>
    </div>

    <div class="grid gap-2">
      <Button @click="submit">分配</Button>
    </div>

    <div v-if="error" class="grid gap-2">
      <Badge variant="destructive">{{ error }}</Badge>
    </div>

    <div v-if="impact" class="grid gap-2">
      <span
        :class="[
          'text-sm',
          impact.overloaded ? 'text-destructive' : 'text-primary',
        ]"
      >
        → 利用率 {{ Math.round(impact.utilization * 100) }}%{{ impact.overloaded ? " ⚠过载" : "" }}
      </span>
    </div>
  </div>
</template>
