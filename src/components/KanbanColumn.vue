<script setup lang="ts">
import { ref } from "vue";
import { NText } from "naive-ui";
import type { KanbanTask, TaskStatus } from "../types";
import TaskCard from "./TaskCard.vue";

const props = defineProps<{ status: TaskStatus; tasks: KanbanTask[] }>();
const emit = defineEmits<{
  (e: "drop", status: TaskStatus): void;
  (e: "dragstart-card", id: number): void;
}>();
const dragging = ref(false);

function onDrop() { dragging.value = false; emit("drop", props.status); }
</script>

<template>
  <div
    class="kanban-column"
    :class="{ 'kanban-column--dragging': dragging }"
    @dragover.prevent="dragging = true"
    @dragleave="dragging = false"
    @drop="onDrop"
  >
    <n-text strong class="kanban-column__header">
      {{ status }} ({{ tasks.length }})
    </n-text>
    <TaskCard
      v-for="t in tasks"
      :key="t.id"
      :task="t"
      @dragstart="(id: number) => emit('dragstart-card', id)"
    />
  </div>
</template>

<style scoped>
.kanban-column {
  width: 240px;
  min-width: 240px;
  background: #f5f5f8;
  border-radius: 8px;
  padding: 8px;
  height: 100%;
  overflow-y: auto;
  transition: background 0.2s ease;
}
.kanban-column--dragging {
  background: #e8f0e8;
}
.kanban-column__header {
  display: block;
  text-transform: capitalize;
  margin-bottom: 8px;
  padding: 4px;
}
</style>
