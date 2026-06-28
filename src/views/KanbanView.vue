<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import KanbanColumn from "../components/KanbanColumn.vue";
import type { TaskStatus } from "../types";

const tasks = useTasksStore();
const projects = useProjectsStore();
const draggingId = ref<number | null>(null);

watchEffect(async () => {
  if (projects.current) await tasks.load(projects.current);
});

function onDrop(status: TaskStatus) {
  if (draggingId.value == null) return;
  tasks.moveStatus(draggingId.value, status);
  draggingId.value = null;
}
</script>

<template>
  <div>
    <h2 style="margin-top:0">看板 / Kanban</h2>
    <div style="display:flex; gap:12px; align-items:flex-start">
      <KanbanColumn
        v-for="col in tasks.columns"
        :key="col"
        :status="col"
        :tasks="tasks.byStatus(col)"
        @drop="onDrop"
        @dragstart-card="(id: number) => (draggingId = id)"
      />
    </div>
  </div>
</template>