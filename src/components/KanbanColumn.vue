<script setup lang="ts">
import { ref } from "vue";
import type { KanbanTask, TaskStatus } from "@/types";
import TaskCard from "@/components/TaskCard.vue";

const props = defineProps<{ status: TaskStatus; tasks: KanbanTask[] }>();
const emit = defineEmits<{
  (e: "drop", status: TaskStatus): void;
  (e: "dragstart-card", id: number): void;
  (e: "delete-card", id: number): void;
  (e: "edit-card", task: KanbanTask): void;
}>();
const dragging = ref(false);

function onDrop() { dragging.value = false; emit("drop", props.status); }
</script>

<template>
  <div
    class="kanban-column flex flex-col h-full w-[260px] min-w-[260px] rounded-lg p-2 overflow-hidden bg-muted transition-colors duration-200"
    :class="{ 'bg-accent': dragging }"
    @dragover.prevent="dragging = true"
    @dragleave="dragging = false"
    @drop="onDrop"
  >
    <span class="kanban-column__header font-semibold shrink-0">
      {{ status }} ({{ tasks.length }})
    </span>
    <div class="flex-1 min-h-0 overflow-y-auto -mx-1 px-1">
      <TaskCard
        v-for="t in tasks"
        :key="t.id"
        :task="t"
        @dragstart="(id: number) => emit('dragstart-card', id)"
        @delete="(id: number) => emit('delete-card', id)"
        @edit="(task: KanbanTask) => emit('edit-card', task)"
      />
    </div>
  </div>
</template>

<style scoped>
.kanban-column__header {
  display: block;
  text-transform: capitalize;
  margin-bottom: 8px;
  padding: 4px;
}
</style>
