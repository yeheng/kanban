<script setup lang="ts">
import { NText, NTag, NButton, NPopconfirm } from "naive-ui";
import type { KanbanTask } from "../types";
import { useUnitStore } from "../stores/unit";
defineProps<{ task: KanbanTask }>();
const emit = defineEmits<{
  (e: "dragstart", id: number): void;
  (e: "delete", id: number): void;
  (e: "edit", task: KanbanTask): void;
}>();
const unit = useUnitStore();
</script>

<template>
  <div
    class="task-card"
    :class="{ 'task-card--segment': task.parent_task_id != null }"
    :style="{ marginLeft: task.parent_task_id != null ? '16px' : '0' }"
    draggable="true"
    @dragstart="emit('dragstart', task.id)"
    @click="emit('edit', task)"
  >
    <div class="task-card__header">
      <n-text strong class="task-card__title">{{ task.title }}</n-text>
      <n-popconfirm @positive-click="emit('delete', task.id)">
        <template #trigger>
          <n-button
            size="tiny"
            type="error"
            quaternary
            circle
            class="task-card__delete"
            @click.stop
          >×</n-button>
        </template>
        确定删除此任务吗？
      </n-popconfirm>
    </div>
    <div class="task-card__meta">
      <n-tag size="tiny" :bordered="false">{{ unit.formatPd(task.estimate_pd) }}</n-tag>
      <n-tag v-if="task.skill_count" size="tiny" :bordered="false" type="info">
        {{ task.skill_count }} skill(s)
      </n-tag>
      <n-tag v-if="task.is_long_term" size="tiny" :bordered="false" type="warning">长期</n-tag>
      <n-tag v-if="task.segment_kind" size="tiny" :bordered="false" type="warning">{{ task.segment_kind }}</n-tag>
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
.task-card__header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}
.task-card__title {
  flex: 1;
}
.task-card__delete {
  flex-shrink: 0;
  margin-left: 4px;
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
