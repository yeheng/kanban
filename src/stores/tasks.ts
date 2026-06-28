import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "../api";
import type { KanbanTask, TaskStatus } from "../types";

const COLUMNS: TaskStatus[] = ["todo", "in_progress", "blocked", "review", "done"];

export const useTasksStore = defineStore("tasks", () => {
  const tasks = ref<KanbanTask[]>([]);

  async function load(projectId: number) { tasks.value = await api.kanbanTasks(projectId); }
  async function moveStatus(taskId: number, status: TaskStatus) {
    const t = tasks.value.find((x) => x.id === taskId);
    if (!t) return;
    const prev = t.status;
    t.status = status;
    try { await api.setTaskStatus(taskId, status); }
    catch (e) { t.status = prev; throw e; }
  }
  function byStatus(status: TaskStatus): KanbanTask[] {
    return tasks.value.filter((t) => t.status === status).sort((a, b) => a.sort_order - b.sort_order);
  }
  const columns = computed(() => COLUMNS);

  return { tasks, columns, load, moveStatus, byStatus };
});