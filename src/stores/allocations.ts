import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { AllocationView } from "../types";

export const useAllocationsStore = defineStore("allocations", () => {
  const items = ref<AllocationView[]>([]);
  async function load(projectId: number) { items.value = await api.listAllocations(projectId); }
  async function create(resourceId: number, taskId: number, start: string, end: string, percent: number) {
    await api.createAllocation(resourceId, taskId, start, end, percent);
  }
  return { items, load, create };
});
