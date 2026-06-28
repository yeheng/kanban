import { describe, it, expect, vi, beforeEach } from "vitest";
import { api } from "./index";

beforeEach(() => {
  vi.restoreAllMocks();
});

function mockFetch(response: { ok: boolean; status: number; json?: unknown; text?: string }) {
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: response.ok,
    status: response.status,
    json: async () => response.json,
    text: async () => response.text ?? "",
  } as Response);
}

describe("api client", () => {
  it("createProject sends snake_case budget_pd", async () => {
    mockFetch({ ok: true, status: 201, json: 7 });
    const id = await api.createProject("Atlas", 3, 40);
    expect(id).toBe(7);
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/projects",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ name: "Atlas", priority: 3, budget_pd: 40 }),
      })
    );
  });

  it("createTask maps camelCase to snake_case body", async () => {
    mockFetch({ ok: true, status: 201, json: 1 });
    await api.createTask({
      projectId: 2,
      title: "T",
      estimatePd: 5,
      start: null,
      end: null,
      skillReqs: [[1, 3, true, 1]],
      tagIds: [9],
    });
    const body = JSON.parse((globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls[0][1].body);
    expect(body.project_id).toBe(2);
    expect(body.estimate_pd).toBe(5);
    expect(body.is_long_term).toBe(false);
    expect(body.skill_reqs).toEqual([[1, 3, true, 1]]);
  });

  it("setTaskStatus calls PATCH endpoint", async () => {
    mockFetch({ ok: true, status: 204 });
    await api.setTaskStatus(1, "done");
    expect(globalThis.fetch).toHaveBeenCalledWith(
      "/api/tasks/1/status",
      expect.objectContaining({
        method: "PATCH",
        body: JSON.stringify({ status: "done" }),
      })
    );
  });
});
