import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "../api";
import type { KanbanTask, TaskStatus } from "../types";

const COLUMNS: TaskStatus[] = ["todo", "in_progress", "blocked", "review", "done"];

export const useTasksStore = defineStore("tasks", () => {
  const tasks = ref<KanbanTask[]>([]);
  const projectId = ref<number | null>(null);

  async function load(pid: number) { projectId.value = pid; tasks.value = await api.kanbanTasks(pid); }
  async function create(args: {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: [number, number, boolean, number][]; tagIds: number[];
    description?: string | null;
  }) {
    await api.createTask(args);
    await load(args.projectId);
  }
  async function update(id: number, args: {
    title: string; estimatePd: number; start: string | null; end: string | null;
    description?: string | null;
  }, pid: number) {
    await api.updateTask(id, args);
    await load(pid);
  }
  async function addDependency(taskId: number, predecessorId: number, lagDays?: number) {
    await api.addDependency(taskId, predecessorId, lagDays);
  }
  async function remove(id: number, pid: number) {
    await api.deleteTask(id);
    await load(pid);
  }
  async function moveStatus(taskId: number, status: TaskStatus) {
    const t = tasks.value.find((x) => x.id === taskId);
    if (!t) return;
    const prev = t.status;
    t.status = status;
    try { await api.setTaskStatus(taskId, status); }
    catch (e) { t.status = prev; throw e; }
    if (projectId.value != null) await api.kanbanTasks(projectId.value).then((rows) => { tasks.value = rows; });
  }
  function byStatus(status: TaskStatus): KanbanTask[] {
    return tasks.value.filter((t) => t.status === status).sort((a, b) => a.sort_order - b.sort_order);
  }
  const columns = computed(() => COLUMNS);

  return { tasks, columns, load, create, update, addDependency, remove, moveStatus, byStatus };
});
