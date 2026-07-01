import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useOptimizationStore } from "./optimization";
import { api } from "../api";

vi.mock("../api", () => ({
  api: {
    runOptimization: vi.fn(),
    listOptimizationRuns: vi.fn().mockResolvedValue({ rows: [], total: 0 }),
    getOptimizationRun: vi.fn(),
    listSuggestions: vi.fn().mockResolvedValue([]),
    rerun: vi.fn(),
    setSuggestionStatus: vi.fn().mockResolvedValue(undefined),
    applySolution: vi.fn().mockResolvedValue(undefined),
    rejectSolution: vi.fn().mockResolvedValue(undefined),
  },
}));
beforeEach(() => { setActivePinia(createPinia()); });

describe("optimization store", () => {
  it("runs with current weights and stores plan", async () => {
    vi.mocked(api.runOptimization).mockResolvedValue({ run_id: 7, plan: { solution: { run_id: 7, assignments: [], unscheduled: [], metrics: { overall: 80, skill_fit: 80, scheduled_ratio: 100, fairness: 0 } }, explanation_md: "ok" } });
    const s = useOptimizationStore();
    await s.run(5);
    expect(s.current?.run_id).toBe(7);
    expect(vi.mocked(api.runOptimization)).toHaveBeenCalledWith(5, s.weights);
  });
  it("normalize makes weights sum to 1", () => {
    const s = useOptimizationStore();
    s.weights = { skill_fit: 1, balance: 1, budget: 2 };
    s.normalize();
    const sum = s.weights.skill_fit + s.weights.balance + s.weights.budget;
    expect(Math.abs(sum - 1)).toBeLessThan(1e-9);
  });

  it("toggleSuggestion updates local status", async () => {
    const opt = useOptimizationStore();
    opt.suggestions = [
      { id: 1, suggestion: { kind: "widen_window", task_id: 5, new_start: "2026-07-01", new_end: "2026-07-20" }, rationale_md: "x", status: "proposed" },
    ];
    await opt.toggleSuggestion(1, true);
    expect(opt.suggestions[0].status).toBe("accepted");
    await opt.toggleSuggestion(1, false);
    expect(opt.suggestions[0].status).toBe("skipped");
  });

  it("rerun sets compareTarget to the prior current", async () => {
    const opt = useOptimizationStore();
    const parent = { run_id: 7, plan: { solution: { run_id: 7, assignments: [], unscheduled: [], metrics: { overall: 50, skill_fit: 0, scheduled_ratio: 0, fairness: 0 }, status: "feasible" as const }, explanation_md: "" } };
    const child = { run_id: 8, plan: { solution: { run_id: 8, assignments: [], unscheduled: [], metrics: { overall: 70, skill_fit: 0, scheduled_ratio: 0, fairness: 0 }, status: "feasible" as const }, explanation_md: "" } };
    opt.current = parent as any;
    (api.rerun as any).mockResolvedValueOnce(child);
    await opt.rerun(7, [1]);
    expect(opt.compareTarget).toEqual(parent);
    expect(opt.current).toEqual(child);
  });
});
