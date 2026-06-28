import type { Project, KanbanTask, Skill, Tag, Resource, TaskStatus } from "../types";

export type SkillReq = [number, number, boolean, number];

const BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? "";

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {};
  const init: RequestInit = { method };
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  init.headers = headers;

  const res = await fetch(`${BASE}${path}`, init);
  if (!res.ok) {
    const text = await res.text().catch(() => "request failed");
    throw new Error(text);
  }
  if (res.status === 204) {
    return undefined as T;
  }
  return res.json() as Promise<T>;
}

export const api = {
  listProjects: (): Promise<Project[]> => request("GET", "/api/projects"),
  createProject: (name: string, priority: number, budgetPd: number): Promise<number> =>
    request("POST", "/api/projects", { name, priority, budget_pd: budgetPd }),

  listSkills: (): Promise<Skill[]> => request("GET", "/api/skills"),
  ensureSkill: (name: string): Promise<number> => request("POST", "/api/skills", { name }),
  listTags: (): Promise<Tag[]> => request("GET", "/api/tags"),
  ensureTag: (name: string, color: string | null): Promise<number> =>
    request("POST", "/api/tags", { name, color }),

  createTask: (args: {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
  }): Promise<number> =>
    request("POST", "/api/tasks", {
      project_id: args.projectId,
      title: args.title,
      estimate_pd: args.estimatePd,
      start: args.start,
      end: args.end,
      skill_reqs: args.skillReqs,
      tag_ids: args.tagIds,
      description: null,
      is_long_term: false,
      sort_order: 0,
    }),
  setTaskStatus: (id: number, status: TaskStatus): Promise<void> =>
    request("PATCH", `/api/tasks/${id}/status`, { status }),
  kanbanTasks: (projectId: number): Promise<KanbanTask[]> =>
    request("GET", `/api/projects/${projectId}/kanban`),

  listResources: (): Promise<Resource[]> => request("GET", "/api/resources"),
  createResource: (name: string, email: string | null): Promise<number> =>
    request("POST", "/api/resources", { name, email }),
};
