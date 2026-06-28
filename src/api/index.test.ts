import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { api } from "./index";

beforeEach(() => vi.mocked(invoke).mockReset());

describe("api client", () => {
  it("createProject passes snake_case budgetPd", async () => {
    vi.mocked(invoke).mockResolvedValue(7);
    const id = await api.createProject("Atlas", 3, 40);
    expect(id).toBe(7);
    expect(invoke).toHaveBeenCalledWith("create_project", { name: "Atlas", priority: 3, budgetPd: 40 });
  });

  it("createTask maps camelCase to the command args", async () => {
    vi.mocked(invoke).mockResolvedValue(1);
    await api.createTask({ projectId: 2, title: "T", estimatePd: 5, start: null, end: null, skillReqs: [[1, 3, true, 1]], tagIds: [9] });
    const args = vi.mocked(invoke).mock.calls[0][1] as Record<string, unknown>;
    expect(args.projectId).toBe(2);
    expect(args.estimatePd).toBe(5);
    expect(args.isLongTerm).toBe(false);
  });

  it("setTaskStatus calls the command", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await api.setTaskStatus(1, "done");
    expect(invoke).toHaveBeenCalledWith("set_task_status", { id: 1, status: "done" });
  });
});