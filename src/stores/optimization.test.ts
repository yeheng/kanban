import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useOptimizationStore } from "./optimization";
import { api } from "../api";

vi.mock("../api", () => ({ api: { runOptimization: vi.fn(), listOptimizationRuns: vi.fn(), applySolution: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); });

describe("optimization store", () => {
  it("runs with current weights and stores plan", async () => {
    vi.mocked(api.runOptimization).mockResolvedValue({ run_id: 7, plan: { solution: { run_id: 7, assignments: [], unscheduled: [], metrics: { overall: 80, skill_fit: 80, utilization: 100, fairness: 0 } }, explanation_md: "ok" } });
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
});
