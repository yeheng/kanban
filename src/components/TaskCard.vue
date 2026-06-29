<script setup lang="ts">
import { NText, NTag } from "naive-ui";
import type { KanbanTask } from "../types";
defineProps<{ task: KanbanTask }>();
const emit = defineEmits<{ (e: "dragstart", id: number): void }>();
</script>

<template>
  <div
    class="task-card"
    draggable="true"
    @dragstart="emit('dragstart', task.id)"
  >
    <n-text strong class="task-card__title">{{ task.title }}</n-text>
    <div class="task-card__meta">
      <n-tag size="tiny" :bordered="false">{{ task.estimate_pd }} PD</n-tag>
      <n-tag v-if="task.skill_count" size="tiny" :bordered="false" type="info">
        {{ task.skill_count }} skill(s)
      </n-tag>
    </div>
    <n-text v-if="task.assignee" depth="3" class="task-card__assignee">
      @{{ task.assignee }}
    </n-text>
  </div>
</template>

<style scoped>
.task-card {
  background: #fff;
  border: 1px solid #e0e0e6;
  border-radius: 6px;
  padding: 8px 10px;
  margin-bottom: 8px;
  cursor: grab;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  transition: box-shadow 0.2s ease;
}
.task-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
.task-card__title {
  display: block;
}
.task-card__meta {
  display: flex;
  gap: 4px;
  margin-top: 4px;
}
.task-card__assignee {
  font-size: 12px;
  margin-top: 2px;
}
</style>
