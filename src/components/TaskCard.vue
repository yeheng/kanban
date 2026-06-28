<script setup lang="ts">
import type { KanbanTask } from "../types";
defineProps<{ task: KanbanTask }>();
const emit = defineEmits<{ (e: "dragstart", id: number): void }>();
</script>

<template>
  <div
    class="card"
    draggable="true"
    @dragstart="emit('dragstart', task.id)"
  >
    <div class="title">{{ task.title }}</div>
    <div class="meta">
      <span>{{ task.estimate_pd }} PD</span>
      <span v-if="task.skill_count">· {{ task.skill_count }} skill(s)</span>
    </div>
    <div v-if="task.assignee" class="assignee">@{{ task.assignee }}</div>
  </div>
</template>

<style scoped>
.card { background:#fff; border:1px solid #e0e0e6; border-radius:6px; padding:8px 10px; margin-bottom:8px; cursor:grab; box-shadow:0 1px 2px rgba(0,0,0,.04); }
.title { font-weight:600; }
.meta { font-size:12px; color:#888; margin-top:4px; }
.assignee { font-size:12px; color:#2080f0; margin-top:2px; }
</style>