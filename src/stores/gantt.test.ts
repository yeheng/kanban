import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useGanttStore } from "./gantt";
import { api } from "../api";

vi.mock("../api", () => ({ api: { ganttProject: vi.fn(), ganttResource: vi.fn(), dependenciesForProject: vi.fn(), updateAllocation: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); });

describe("gantt store", () => {
  it("loads project bars + deps", async () => {
    vi.mocked(api.ganttProject).mockResolvedValue([{ allocation_id: 1, resource_id: 1, resource_name: "A", task_id: 10, task_title: "T", project_id: 5, project_name: "P", start_date: "2026-06-29", end_date: "2026-07-03", percent: 0.5, status: "planned", source: "manual" }]);
    vi.mocked(api.dependenciesForProject).mockResolvedValue([]);
    const s = useGanttStore(); s.mode = "project"; s.focusId = 5;
    await s.load();
    expect(s.bars.length).toBe(1);
    expect(s.deps.length).toBe(0);
  });

  it("clears deps in resource mode", async () => {
    vi.mocked(api.ganttResource).mockResolvedValue([]);
    const s = useGanttStore(); s.mode = "resource"; s.focusId = 2; s.deps = [{ task_id: 1, predecessor_id: 2, lag_days: 0, dep_type: "FS" }];
    await s.load();
    expect(s.deps.length).toBe(0);
  });
});
