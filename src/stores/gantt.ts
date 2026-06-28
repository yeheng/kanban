import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { GanttBar, DepEdge } from "../types";

export const useGanttStore = defineStore("gantt", () => {
  const bars = ref<GanttBar[]>([]);
  const deps = ref<DepEdge[]>([]);
  const mode = ref<"project" | "resource">("project");
  const focusId = ref<number | null>(null); // project_id or resource_id depending on mode

  async function load() {
    if (mode.value === "project" && focusId.value != null) {
      [bars.value, deps.value] = await Promise.all([
        api.ganttProject(focusId.value), api.dependenciesForProject(focusId.value)]);
    } else if (mode.value === "resource" && focusId.value != null) {
      bars.value = await api.ganttResource(focusId.value);
      deps.value = [];
    }
  }
  async function moveOrResize(allocationId: number, start: string, end: string, percent: number) {
    await api.updateAllocation(allocationId, start, end, percent);
    await load();
  }
  return { bars, deps, mode, focusId, load, moveOrResize };
});
