import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useRefreshStore } from "./refresh";
import { useOptimizationStore } from "./optimization";
import { useAllocationsStore } from "./allocations";
import { api } from "../api";

// Minimal api mock: accept() calls applySolution + listOptimizationRuns; allocation writes
// call createAllocation + listAllocations. We only assert refresh-bus side effects here.
vi.mock("../api", () => ({
  api: {
    applySolution: vi.fn().mockResolvedValue(undefined),
    listOptimizationRuns: vi.fn().mockResolvedValue([]),
    createAllocation: vi.fn().mockResolvedValue(1),
    listAllocations: vi.fn().mockResolvedValue([]),
    rejectSolution: vi.fn().mockResolvedValue(undefined),
  },
}));
beforeEach(() => { setActivePinia(createPinia()); });

describe("refresh store", () => {
  it("bump increments only the named scopes", () => {
    const r = useRefreshStore();
    expect(r.version.workload).toBe(0);
    expect(r.version.gantt).toBe(0);
    r.bump("workload", "gantt");
    expect(r.version.workload).toBe(1);
    expect(r.version.gantt).toBe(1);
    expect(r.version.kanban).toBe(0); // untouched
  });

  it("AI accept bumps allocations/workload/gantt/kanban/calendar", async () => {
    const r = useRefreshStore();
    const opt = useOptimizationStore();
    await opt.accept(42);
    for (const scope of ["allocations", "workload", "gantt", "kanban", "calendar"] as const) {
      expect(r.version[scope]).toBe(1, `${scope} bumped after accept`);
    }
  });

  it("allocation create bumps the allocation-derived scopes", async () => {
    const r = useRefreshStore();
    const alloc = useAllocationsStore();
    await alloc.create(1, 10, "2026-07-01", "2026-07-05", 0.5, 1);
    expect(r.version.allocations).toBe(1);
    expect(r.version.workload).toBe(1);
    expect(r.version.gantt).toBe(1);
    expect(r.version.kanban).toBe(1);
    expect(r.version.calendar).toBe(1);
  });
});
