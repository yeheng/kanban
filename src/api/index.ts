import type { Project, KanbanTask, Skill, Tag, Resource, TaskStatus, ResourceSummary, TeamSummary, ProjectBurn, Thresholds, AllocationView, Task, Team, TeamMember, Holiday, WeekTemplate, GanttBar, DepEdge, DayOccupancy, ObjectiveWeights, RunResult, RunRow } from "../types";

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

  // ---- Phase 2: workload ----
  resourceSummary: (resourceId: number, start: string, end: string): Promise<ResourceSummary> =>
    request("GET", `/api/workload/resources/${resourceId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  teamSummary: (teamId: number, start: string, end: string): Promise<TeamSummary> =>
    request("GET", `/api/workload/teams/${teamId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  overloads: (start: string, end: string): Promise<ResourceSummary[]> =>
    request("GET", `/api/workload/overloads?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  projectBurn: (projectId: number): Promise<ProjectBurn> =>
    request("GET", `/api/projects/${projectId}/burn`),
  getThresholds: (): Promise<Thresholds> => request("GET", "/api/thresholds"),

  // ---- Phase 2: allocations ----
  createAllocation: (resourceId: number, taskId: number, start: string, end: string, percent: number): Promise<number> =>
    request("POST", "/api/allocations", { resource_id: resourceId, task_id: taskId, start, end, percent }),
  listAllocations: (projectId: number): Promise<AllocationView[]> =>
    request("GET", `/api/projects/${projectId}/allocations`),
  listTasks: (projectId: number): Promise<Task[]> =>
    request("GET", `/api/projects/${projectId}/tasks`),

  // ---- Phase 2: calendar ----
  setGlobalWorkWeek: (week: number[]): Promise<void> =>
    request("POST", "/api/calendar/work-week", { week }),
  listWorkWeeks: (): Promise<WeekTemplate[]> => request("GET", "/api/calendar/work-week"),
  addHoliday: (projectId: number | null, day: string, fraction: number | null, name: string | null): Promise<number> =>
    request("POST", "/api/calendar/holidays", { project_id: projectId, day, fraction, name }),
  listHolidays: (): Promise<Holiday[]> => request("GET", "/api/calendar/holidays"),
  addTimeOff: (resourceId: number, day: string, fraction: number | null, reason: string | null): Promise<number> =>
    request("POST", "/api/calendar/time-off", { resource_id: resourceId, day, fraction, reason }),

  // ---- Phase 2: teams ----
  listTeams: (): Promise<Team[]> => request("GET", "/api/teams"),
  listTeamMembers: (teamId: number): Promise<TeamMember[]> =>
    request("GET", `/api/teams/${teamId}/members`),

  // ---- Phase 3: Gantt + occupancy ----
  ganttProject: (projectId: number): Promise<GanttBar[]> =>
    request("GET", `/api/gantt/projects/${projectId}`),
  ganttResource: (resourceId: number): Promise<GanttBar[]> =>
    request("GET", `/api/gantt/resources/${resourceId}`),
  dependenciesForProject: (projectId: number): Promise<DepEdge[]> =>
    request("GET", `/api/projects/${projectId}/dependencies`),
  dailyOccupancy: (start: string, end: string): Promise<DayOccupancy[]> =>
    request("GET", `/api/occupancy?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  updateAllocation: (id: number, start: string, end: string, percent: number): Promise<void> =>
    request("PUT", `/api/allocations/${id}`, { start, end, percent }),

  // ---- Phase 4: AI optimization ----
  runOptimization: (projectId: number, weights: ObjectiveWeights | null): Promise<RunResult> =>
    request("POST", `/api/optimization/run/${projectId}`, weights ?? undefined),
  listOptimizationRuns: (limit: number | null): Promise<RunRow[]> =>
    request("GET", `/api/optimization/runs${limit != null ? `?limit=${limit}` : ""}`),
  applySolution: (runId: number): Promise<number> =>
    request("POST", `/api/optimization/runs/${runId}/apply`),
  rejectSolution: (runId: number): Promise<void> =>
    request("POST", `/api/optimization/runs/${runId}/reject`),
};
