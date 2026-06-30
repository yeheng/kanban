import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { AllocationView } from "../types";
import { useRefreshStore } from "./refresh";

export const useAllocationsStore = defineStore("allocations", () => {
  const items = ref<AllocationView[]>([]);
  async function load(projectId: number) { items.value = await api.listAllocations(projectId); }
  async function create(resourceId: number, taskId: number, start: string, end: string, percent: number, projectId?: number) {
    await api.createAllocation(resourceId, taskId, start, end, percent);
    if (projectId != null) await load(projectId);
    // An allocation write invalidates every allocation-derived view (design G4).
    useRefreshStore().bump("allocations", "workload", "gantt", "kanban", "calendar");
  }
  async function update(id: number, start: string, end: string, percent: number, projectId: number) {
    await api.updateAllocation(id, start, end, percent);
    await load(projectId);
    useRefreshStore().bump("allocations", "workload", "gantt", "kanban", "calendar");
  }
  async function remove(id: number, projectId: number) {
    await api.deleteAllocation(id);
    await load(projectId);
    useRefreshStore().bump("allocations", "workload", "gantt", "kanban", "calendar");
  }
  return { items, load, create, update, remove };
});
