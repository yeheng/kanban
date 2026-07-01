<script setup lang="ts">
import { computed, ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { NumberField, NumberFieldContent, NumberFieldDecrement, NumberFieldIncrement, NumberFieldInput } from "@/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  useKanbanTasksQuery,
  useUpdateTaskMutation,
  useDeleteTaskMutation,
  useSetTaskStatusMutation,
  useAddDependencyMutation,
} from "@/services/api/tasks.api";
import { useProjectsStore } from "@/stores/projects";
import KanbanColumn from "@/components/KanbanColumn.vue";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { fmtDate, parseDate } from "@/utils/date";
import type { KanbanTask, TaskStatus } from "@/types";

const COLUMNS: TaskStatus[] = ["todo", "in_progress", "blocked", "review", "done"];

const projects = useProjectsStore();
const kanbanQuery = useKanbanTasksQuery(computed(() => projects.current));
const updateTask = useUpdateTaskMutation();
const deleteTask = useDeleteTaskMutation();
const setTaskStatus = useSetTaskStatusMutation();
const addDependency = useAddDependencyMutation();
const draggingId = ref<number | null>(null);

const tasks = computed(() => kanbanQuery.data.value ?? []);

// Edit modal state
const editVisible = ref(false);
const editing = ref<KanbanTask | null>(null);
const editTitle = ref("");
const editEstimate = ref(0);
const editDateRange = ref<[number, number] | null>(null);
const editDescription = ref("");
const depPredecessor = ref<number | null>(null);
const depLag = ref(0);
const editError = ref<string | null>(null);

/** Pull the human-readable `detail` out of the backend's `{code,detail}` error body. */
function errText(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  try {
    const j = JSON.parse(raw);
    if (j && typeof j.detail === "string") return j.detail;
  } catch { /* not a JSON error body */ }
  return raw;
}

const otherTasks = computed(() =>
  tasks.value.filter((t) => t.id !== editing.value?.id),
);
const predecessorOptions = computed(() =>
  otherTasks.value.map((t) => ({ label: t.title, value: t.id })),
);

function byStatus(status: TaskStatus): KanbanTask[] {
  return tasks.value.filter((t) => t.status === status).sort((a, b) => a.sort_order - b.sort_order);
}

function onDrop(status: TaskStatus) {
  if (draggingId.value == null) return;
  const task = tasks.value.find((t) => t.id === draggingId.value);
  if (!task) return;
  const prevStatus = task.status;
  task.status = status; // optimistic
  setTaskStatus.mutate(
    { id: draggingId.value, status, projectId: projects.current ?? undefined },
    {
      onError: () => {
        task.status = prevStatus;
      },
    },
  );
  draggingId.value = null;
}

function onEdit(task: KanbanTask) {
  editing.value = task;
  editTitle.value = task.title;
  editEstimate.value = task.estimate_pd;
  const start = parseDate(task.start_date);
  const end = parseDate(task.end_date);
  editDateRange.value = start != null && end != null ? [start, end] : null;
  editDescription.value = task.description ?? "";
  depPredecessor.value = null;
  depLag.value = 0;
  editError.value = null;
  editVisible.value = true;
}

async function saveEdit() {
  if (!editing.value || !projects.current) return;
  editError.value = null;
  try {
    await updateTask.mutateAsync({
      id: editing.value.id,
      projectId: projects.current,
      title: editTitle.value,
      estimatePd: editEstimate.value,
      start: editDateRange.value ? fmtDate(editDateRange.value[0]) : null,
      end: editDateRange.value ? fmtDate(editDateRange.value[1]) : null,
      description: editDescription.value || null,
    });
    if (depPredecessor.value != null) {
      await addDependency.mutateAsync({
        taskId: editing.value.id,
        predecessorId: depPredecessor.value,
        lagDays: depLag.value,
        projectId: projects.current,
      });
    }
    editVisible.value = false;
  } catch (e: unknown) {
    editError.value = errText(e);
  }
}

async function onDelete(id: number) {
  await deleteTask.mutateAsync({ id, projectId: projects.current ?? undefined });
}

function onPredecessorChange(value: unknown) {
  depPredecessor.value = value == null ? null : (value as number);
}
</script>

<template>
  <div class="h-full flex flex-col">
    <h2 class="text-2xl font-bold mt-0 mb-4">看板 / Kanban</h2>
    <div v-if="tasks.length" class="flex-1 flex items-start gap-3 min-h-0 overflow-x-auto pb-2">
      <KanbanColumn
        v-for="col in COLUMNS"
        :key="col"
        :status="col"
        :tasks="byStatus(col)"
        @drop="onDrop"
        @dragstart-card="(id: number) => (draggingId = id)"
        @delete-card="onDelete"
        @edit-card="onEdit"
      />
    </div>
    <div v-else class="text-muted-foreground">
      暂无任务，请到项目页面创建任务
      <div class="mt-4">
        <Button @click="$router.push('/projects')">去创建任务</Button>
      </div>
    </div>

    <Dialog v-model:open="editVisible">
      <DialogContent class="max-w-lg">
        <DialogHeader>
          <DialogTitle>编辑任务</DialogTitle>
          <DialogDescription />
        </DialogHeader>
        <div v-if="editing" class="grid gap-4">
          <div class="grid gap-2">
            <Label>标题</Label>
            <Input v-model="editTitle" />
          </div>
          <div class="grid gap-2">
            <Label>描述</Label>
            <Textarea v-model="editDescription" :rows="2" placeholder="任务描述 (可选)" />
          </div>
          <div class="grid gap-2">
            <Label>估时 (PD)</Label>
            <NumberField v-model="editEstimate" :min="0">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div v-if="editDateRange" class="grid gap-2">
            <Label>区间</Label>
            <DateRangePicker v-model="editDateRange" />
          </div>
          <div v-else class="grid gap-2">
            <Label>区间</Label>
            <div class="text-sm text-muted-foreground">未设置日期</div>
          </div>
          <div class="grid gap-2">
            <Label>前置任务</Label>
            <div class="flex items-center gap-2 flex-wrap">
              <Select :model-value="depPredecessor ?? undefined" @update:model-value="onPredecessorChange">
                <SelectTrigger class="w-60">
                  <SelectValue placeholder="选择前置任务 (可选)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="opt in predecessorOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</SelectItem>
                </SelectContent>
              </Select>
              <span class="text-muted-foreground">延迟</span>
              <NumberField v-model="depLag" :step="1" class="w-24">
                <NumberFieldContent>
                  <NumberFieldDecrement />
                  <NumberFieldInput />
                  <NumberFieldIncrement />
                </NumberFieldContent>
              </NumberField>
              <span class="text-muted-foreground">天</span>
            </div>
          </div>
          <Alert v-if="editError" variant="destructive">
            <AlertDescription>{{ editError }}</AlertDescription>
          </Alert>
          <div class="flex justify-end gap-2">
            <Button variant="outline" @click="editVisible = false">取消</Button>
            <Button @click="saveEdit">保存</Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>
