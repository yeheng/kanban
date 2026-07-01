<script setup lang="ts">
import { ref } from "vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import type { KanbanTask } from "@/types";
import { useUnitStore } from "@/stores/unit";

defineProps<{ task: KanbanTask }>();
const emit = defineEmits<{
  (e: "dragstart", id: number): void;
  (e: "delete", id: number): void;
  (e: "edit", task: KanbanTask): void;
}>();

const unit = useUnitStore();
const confirmOpen = ref(false);

function onConfirmDelete(id: number) {
  emit("delete", id);
  confirmOpen.value = false;
}
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
      <span class="task-card__title font-medium">{{ task.title }}</span>
      <Dialog v-model:open="confirmOpen">
        <DialogTrigger as-child>
          <Button
            variant="destructive"
            size="icon-xs"
            class="task-card__delete"
            @click.stop="confirmOpen = true"
          >
            ×
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除任务</DialogTitle>
            <DialogDescription>确定删除此任务吗？</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="confirmOpen = false">
              取消
            </Button>
            <Button variant="destructive" @click="onConfirmDelete(task.id)">
              确定
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
    <div class="task-card__meta">
      <Badge variant="default">{{ unit.formatPd(task.estimate_pd) }}</Badge>
      <Badge v-if="task.skill_count" variant="secondary">
        {{ task.skill_count }} skill(s)
      </Badge>
      <Badge v-if="task.is_long_term" variant="outline">长期</Badge>
      <Badge v-if="task.segment_kind" variant="outline">
        {{ task.segment_kind }}
      </Badge>
    </div>
    <span v-if="task.assignee" class="task-card__assignee text-muted-foreground">
      @{{ task.assignee }}
    </span>
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
