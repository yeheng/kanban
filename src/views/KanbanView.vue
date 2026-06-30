<script setup lang="ts">
import { computed, ref, watchEffect } from "vue";
import { NH2, NSpace, NModal, NForm, NFormItem, NInput, NInputNumber, NDatePicker, NButton, NSelect, NEmpty, NText, NTag } from "naive-ui";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import { useRefreshStore } from "../stores/refresh";
import KanbanColumn from "../components/KanbanColumn.vue";
import { fmtDate, parseDate } from "../utils/date";
import type { KanbanTask, TaskStatus } from "../types";

const tasks = useTasksStore();
const projects = useProjectsStore();
const refreshBus = useRefreshStore();
const draggingId = ref<number | null>(null);

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
  tasks.tasks.filter((t) => t.id !== editing.value?.id),
);
const predecessorOptions = computed(() =>
  otherTasks.value.map((t) => ({ label: t.title, value: t.id })),
);

watchEffect(async () => {
  // Reload kanban when the refresh bus bumps it (allocation write / AI accept).
  void refreshBus.version.kanban;
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
    await tasks.update(editing.value.id, {
      title: editTitle.value,
      estimatePd: editEstimate.value,
      start: editDateRange.value ? fmtDate(editDateRange.value[0]) : null,
      end: editDateRange.value ? fmtDate(editDateRange.value[1]) : null,
      description: editDescription.value || null,
    }, projects.current);
    if (depPredecessor.value != null) {
      await tasks.addDependency(editing.value.id, depPredecessor.value, depLag.value);
    }
    editVisible.value = false;
  } catch (e: unknown) {
    // Keep the modal open so the user can correct the input (e.g. a dependency cycle → 422).
    editError.value = errText(e);
  }
}

async function onDelete(id: number) {
  if (!projects.current) return;
  await tasks.remove(id, projects.current);
}
</script>

<template>
  <div>
    <n-h2 style="margin-top: 0">看板 / Kanban</n-h2>
    <n-space v-if="tasks.tasks.length" :size="12" align="start">
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
    <n-empty v-else description="暂无任务，请到项目页面创建任务">
      <template #extra>
        <n-button @click="$router.push('/projects')">去创建任务</n-button>
      </template>
    </n-empty>

    <n-modal
      v-model:show="editVisible"
      preset="card"
      title="编辑任务"
      style="width: 520px"
    >
      <n-form v-if="editing">
        <n-form-item label="标题">
          <n-input v-model:value="editTitle" />
        </n-form-item>
        <n-form-item label="描述">
          <n-input v-model:value="editDescription" type="textarea" :rows="2" placeholder="任务描述 (可选)" />
        </n-form-item>
        <n-form-item label="估时 (PD)">
          <n-input-number v-model:value="editEstimate" :min="0" />
        </n-form-item>
        <n-form-item label="区间">
          <n-date-picker v-model:value="editDateRange" type="daterange" clearable />
        </n-form-item>
        <n-form-item label="前置任务">
          <n-space align="center">
            <n-select
              v-model:value="depPredecessor"
              :options="predecessorOptions"
              placeholder="选择前置任务 (可选)"
              style="width: 240px"
              clearable
            />
            <n-text>延迟</n-text>
            <n-input-number v-model:value="depLag" :step="1" style="width: 100px" />
            <n-text>天</n-text>
          </n-space>
        </n-form-item>
        <n-form-item v-if="editError">
          <n-tag type="error">{{ editError }}</n-tag>
        </n-form-item>
        <n-space justify="end">
          <n-button @click="editVisible = false">取消</n-button>
          <n-button type="primary" @click="saveEdit">保存</n-button>
        </n-space>
      </n-form>
    </n-modal>
  </div>
</template>
