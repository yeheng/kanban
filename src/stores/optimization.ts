import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { RunResult, RunList, ObjectiveWeights } from "../types";
import { useRefreshStore } from "./refresh";

export const useOptimizationStore = defineStore("optimization", () => {
  const current = ref<RunResult | null>(null);
  const history = ref<RunList>({ rows: [], total: 0 });
  const page = ref(1);
  const pageSize = ref(10);
  const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
  const busy = ref(false);

  async function run(projectId: number) {
    busy.value = true;
    try { current.value = await api.runOptimization(projectId, weights.value); }
    finally { busy.value = false; }
  }
  async function loadHistory() {
    const offset = (page.value - 1) * pageSize.value;
    history.value = await api.listOptimizationRuns(offset, pageSize.value);
  }
  async function loadRun(runId: number) { current.value = await api.getOptimizationRun(runId); }
  async function setPage(n: number) {
    const totalPages = Math.max(1, Math.ceil(history.value.total / pageSize.value));
    page.value = Math.max(1, Math.min(n, totalPages));
    await loadHistory();
  }
  async function setPageSize(n: number) {
    pageSize.value = Math.max(1, n);
    page.value = 1;
    await loadHistory();
  }
  async function accept(runId: number) {
    await api.applySolution(runId);
    current.value = null;
    await loadHistory();
    // Applying an AI solution writes allocations, which invalidates every view that caches
    // allocation-derived data (allocations, workload, gantt, kanban, calendar). Bump the shared
    // refresh bus so subscribed views reload instead of showing stale data (design G4).
    useRefreshStore().bump("allocations", "workload", "gantt", "kanban", "calendar");
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
  return { current, history, page, pageSize, weights, busy, run, loadHistory, loadRun, setPage, setPageSize, accept, reject, normalize };
});
