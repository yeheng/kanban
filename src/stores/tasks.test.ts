import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useTasksStore } from "./tasks";
import { api } from "../api";

vi.mock("../api", () => ({
  api: { kanbanTasks: vi.fn(), setTaskStatus: vi.fn() },
}));

beforeEach(() => { setActivePinia(createPinia()); vi.mocked(api.kanbanTasks).mockReset(); vi.mocked(api.setTaskStatus).mockReset(); });

describe("tasks store", () => {
  it("groups tasks by status", async () => {
    vi.mocked(api.kanbanTasks).mockResolvedValue([
      { id: 1, project_id: 1, title: "A", status: "todo", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
      { id: 2, project_id: 1, title: "B", status: "done", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
    ]);
    const s = useTasksStore();
    await s.load(1);
    expect(s.byStatus("todo").map((t) => t.id)).toEqual([1]);
    expect(s.byStatus("done").map((t) => t.id)).toEqual([2]);
  });

  it("moveStatus updates optimistically and rolls back on error", async () => {
    vi.mocked(api.kanbanTasks).mockResolvedValue([
      { id: 1, project_id: 1, title: "A", status: "todo", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
    ]);
    vi.mocked(api.setTaskStatus).mockRejectedValueOnce(new Error("boom"));
    const s = useTasksStore();
    await s.load(1);
    await expect(s.moveStatus(1, "in_progress")).rejects.toThrow("boom");
    expect(s.byStatus("todo")[0].id).toBe(1); // rolled back
  });
});