import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { AllocationView } from "../types";

export const useAllocationsStore = defineStore("allocations", () => {
  const items = ref<AllocationView[]>([]);
  async function load(projectId: number) { items.value = await api.listAllocations(projectId); }
  async function create(resourceId: number, taskId: number, start: string, end: string, percent: number, projectId?: number) {
    await api.createAllocation(resourceId, taskId, start, end, percent);
    if (projectId != null) await load(projectId);
  }
  async function update(id: number, start: string, end: string, percent: number, projectId: number) {
    await api.updateAllocation(id, start, end, percent);
    await load(projectId);
  }
  async function remove(id: number, projectId: number) {
    await api.deleteAllocation(id);
    await load(projectId);
  }
  return { items, load, create, update, remove };
});
