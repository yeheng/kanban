import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { RunResult, RunRow, ObjectiveWeights } from "../types";
import { useAllocationsStore } from "./allocations";
import { useGanttStore } from "./gantt";

export const useOptimizationStore = defineStore("optimization", () => {
  const current = ref<RunResult | null>(null);
  const history = ref<RunRow[]>([]);
  const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
  const busy = ref(false);

  async function run(projectId: number) {
    busy.value = true;
    try { current.value = await api.runOptimization(projectId, weights.value); }
    finally { busy.value = false; }
  }
  async function loadHistory() { history.value = await api.listOptimizationRuns(20); }
  async function accept(runId: number) {
    await api.applySolution(runId);
    current.value = null;
    await loadHistory();
    // Clear dependent stores so stale allocation/gantt data gets reloaded on next visit
    const alloc = useAllocationsStore();
    const gantt = useGanttStore();
    alloc.items = [];
    gantt.bars = [];
    gantt.deps = [];
  }
  async function reject(runId: number) { await api.rejectSolution(runId); current.value = null; await loadHistory(); }
  /** Normalize the three weights to sum to 1 (called on slider change). */
  function normalize() {
    const s = weights.value.skill_fit + weights.value.balance + weights.value.budget;
    if (s > 0) {
      weights.value = {
        skill_fit: weights.value.skill_fit / s,
        balance: weights.value.balance / s,
        budget: weights.value.budget / s,
      };
    }
  }
  return { current, history, weights, busy, run, loadHistory, accept, reject, normalize };
});
