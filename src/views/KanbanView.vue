<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { NH2, NSpace, NModal, NForm, NFormItem, NInput, NInputNumber, NDatePicker, NButton } from "naive-ui";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import KanbanColumn from "../components/KanbanColumn.vue";
import type { KanbanTask, TaskStatus } from "../types";

const tasks = useTasksStore();
const projects = useProjectsStore();
const draggingId = ref<number | null>(null);

// Edit modal state
const editVisible = ref(false);
const editing = ref<KanbanTask | null>(null);
const editTitle = ref("");
const editEstimate = ref(0);
const editDateRange = ref<[number, number] | null>(null);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function parseDate(s: string | null): number | null {
  if (!s) return null;
  return Date.parse(s);
}

watchEffect(async () => {
  if (projects.current) await tasks.load(projects.current);
});

function onDrop(status: TaskStatus) {
  if (draggingId.value == null) return;
  tasks.moveStatus(draggingId.value, status);
  draggingId.value = null;
}

function onEdit(task: KanbanTask) {
  editing.value = task;
  editTitle.value = task.title;
  editEstimate.value = task.estimate_pd;
  const start = parseDate((task as unknown as { start_date?: string }).start_date ?? null);
  const end = parseDate((task as unknown as { end_date?: string }).end_date ?? null);
  editDateRange.value = start != null && end != null ? [start, end] : null;
  editVisible.value = true;
}

async function saveEdit() {
  if (!editing.value || !projects.current) return;
  await tasks.update(editing.value.id, {
    title: editTitle.value,
    estimatePd: editEstimate.value,
    start: editDateRange.value ? fmtDate(editDateRange.value[0]) : null,
    end: editDateRange.value ? fmtDate(editDateRange.value[1]) : null,
  }, projects.current);
  editVisible.value = false;
}

async function onDelete(id: number) {
  if (!projects.current) return;
  await tasks.remove(id, projects.current);
}
</script>

<template>
  <div>
    <n-h2 style="margin-top: 0">看板 / Kanban</n-h2>
    <n-space :size="12" align="start">
      <KanbanColumn
        v-for="col in tasks.columns"
        :key="col"
        :status="col"
        :tasks="tasks.byStatus(col)"
        @drop="onDrop"
        @dragstart-card="(id: number) => (draggingId = id)"
        @delete-card="onDelete"
        @edit-card="onEdit"
      />
    </n-space>

    <n-modal
      v-model:show="editVisible"
      preset="card"
      title="编辑任务"
      style="width: 480px"
    >
      <n-form v-if="editing">
        <n-form-item label="标题">
          <n-input v-model:value="editTitle" />
        </n-form-item>
        <n-form-item label="估时 (PD)">
          <n-input-number v-model:value="editEstimate" :min="0" />
        </n-form-item>
        <n-form-item label="区间">
          <n-date-picker v-model:value="editDateRange" type="daterange" clearable />
        </n-form-item>
        <n-space justify="end">
          <n-button @click="editVisible = false">取消</n-button>
          <n-button type="primary" @click="saveEdit">保存</n-button>
        </n-space>
      </n-form>
    </n-modal>
  </div>
</template>
