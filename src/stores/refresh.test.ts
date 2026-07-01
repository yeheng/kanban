import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useRefreshStore } from "./refresh";
import { useOptimizationStore } from "./optimization";
import { useAllocationsStore } from "./allocations";
import { useGanttStore } from "./gantt";
import { useTasksStore } from "./tasks";
import { useCalendarStore } from "./calendar";
import { api } from "../api";

// Minimal api mock: accept() calls applySolution + listOptimizationRuns; allocation writes
// call createAllocation + listAllocations. We only assert refresh-bus side effects here.
vi.mock("../api", () => ({
  api: {
    applySolution: vi.fn().mockResolvedValue(undefined),
    listOptimizationRuns: vi.fn().mockResolvedValue({ rows: [], total: 0 }),
    createAllocation: vi.fn().mockResolvedValue(1),
    listAllocations: vi.fn().mockResolvedValue([]),
    rejectSolution: vi.fn().mockResolvedValue(undefined),
    updateAllocation: vi.fn().mockResolvedValue(undefined),
    addDependency: vi.fn().mockResolvedValue(undefined),
    addHoliday: vi.fn().mockResolvedValue(undefined),
    listHolidays: vi.fn().mockResolvedValue([]),
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
      expect(r.version[scope], `${scope} bumped after accept`).toBe(1);
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

  it("gantt move/resize bumps the allocation-derived scopes", async () => {
    const r = useRefreshStore();
    const gantt = useGanttStore();
    await gantt.moveOrResize(1, "2026-07-01", "2026-07-05", 0.5);
    for (const scope of ["allocations", "workload", "gantt", "kanban", "calendar"] as const) {
      expect(r.version[scope], `${scope} bumped after gantt move`).toBe(1);
    }
  });

  it("addDependency bumps only the gantt scope", async () => {
    const r = useRefreshStore();
    const tasks = useTasksStore();
    await tasks.addDependency(2, 1, 0);
    expect(r.version.gantt).toBe(1);
    expect(r.version.kanban).toBe(0); // untouched
    expect(r.version.allocations).toBe(0);
  });

  it("calendar mutation bumps workload + calendar", async () => {
    const r = useRefreshStore();
    const cal = useCalendarStore();
    await cal.addHoliday("2026-07-01", 1.0, "H");
    expect(r.version.workload).toBe(1);
    expect(r.version.calendar).toBe(1);
    expect(r.version.gantt).toBe(0); // a holiday doesn't move bars
    expect(r.version.allocations).toBe(0);
  });
});
