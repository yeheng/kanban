import type { Project, KanbanTask, Skill, Tag, Resource, ResourceSkill, ResourceTag, TaskStatus, ResourceSummary, TeamSummary, ProjectBurn, Thresholds, AllocationView, Task, Team, TeamMember, TeamOverride, TimeOff, Holiday, WeekTemplate, GanttBar, DepEdge, DayOccupancy, ObjectiveWeights, RunResult, RunList, Settings } from "../types";

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
  updateProject: (id: number, args: {
    name: string; priority: number; budgetPd: number;
    description?: string | null; start?: string | null; end?: string | null;
  }): Promise<void> =>
    request("PATCH", `/api/projects/${id}`, {
      name: args.name, description: args.description ?? null,
      start: args.start ?? null, end: args.end ?? null,
      priority: args.priority, budget_pd: args.budgetPd,
    }),
  setProjectStatus: (id: number, status: string): Promise<void> =>
    request("PATCH", `/api/projects/${id}/status`, { status }),
  deleteProject: (id: number): Promise<void> =>
    request("DELETE", `/api/projects/${id}`),

  listSkills: (): Promise<Skill[]> => request("GET", "/api/skills"),
  ensureSkill: (name: string): Promise<number> => request("POST", "/api/skills", { name }),
  listTags: (): Promise<Tag[]> => request("GET", "/api/tags"),
  ensureTag: (name: string, color: string | null): Promise<number> =>
    request("POST", "/api/tags", { name, color }),

  createTask: (args: {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }): Promise<number> =>
    request("POST", "/api/tasks", {
      project_id: args.projectId,
      title: args.title,
      estimate_pd: args.estimatePd,
      start: args.start,
      end: args.end,
      skill_reqs: args.skillReqs,
      tag_ids: args.tagIds,
      description: args.description ?? null,
      is_long_term: args.isLongTerm ?? false,
      parent_task_id: args.parentTaskId ?? null,
      segment_kind: args.segmentKind ?? null,
      sort_order: 0,
    }),
  updateTask: (id: number, args: {
    title: string; estimatePd: number;
    start: string | null; end: string | null;
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }): Promise<void> =>
    request("PATCH", `/api/tasks/${id}`, {
      title: args.title,
      description: args.description ?? null,
      estimate_pd: args.estimatePd,
      start: args.start,
      end: args.end,
      is_long_term: args.isLongTerm ?? false,
      parent_task_id: args.parentTaskId ?? null,
      segment_kind: args.segmentKind ?? null,
    }),
  deleteTask: (id: number): Promise<void> =>
    request("DELETE", `/api/tasks/${id}`),
  setTaskStatus: (id: number, status: TaskStatus): Promise<void> =>
    request("PATCH", `/api/tasks/${id}/status`, { status }),
  addDependency: (taskId: number, predecessorId: number, lagDays?: number): Promise<void> =>
    request("POST", `/api/tasks/${taskId}/dependencies`, { predecessor_id: predecessorId, lag_days: lagDays ?? 0, dep_type: "finish_to_start" }),
  kanbanTasks: (projectId: number): Promise<KanbanTask[]> =>
    request("GET", `/api/projects/${projectId}/kanban`),

  listResources: (): Promise<Resource[]> => request("GET", "/api/resources"),
  createResource: (name: string, email: string | null): Promise<number> =>
    request("POST", "/api/resources", { name, email }),
  updateResource: (id: number, args: {
    name: string; email: string | null;
    availableFrom?: string | null; availableTo?: string | null;
    dailyCapacityPd?: number | null; dailyRatePd?: number | null;
  }): Promise<void> =>
    request("PATCH", `/api/resources/${id}`, {
      name: args.name, email: args.email,
      available_from: args.availableFrom ?? null, available_to: args.availableTo ?? null,
      daily_capacity_pd: args.dailyCapacityPd ?? null, daily_rate_pd: args.dailyRatePd ?? null,
    }),
  deleteResource: (id: number): Promise<void> =>
    request("DELETE", `/api/resources/${id}`),
  getResourceSkills: (id: number): Promise<ResourceSkill[]> =>
    request("GET", `/api/resources/${id}/skills`),
  setResourceSkills: (id: number, skills: [number, number][]): Promise<void> =>
    request("PUT", `/api/resources/${id}/skills`, { skills }),
  getResourceTags: (id: number): Promise<ResourceTag[]> =>
    request("GET", `/api/resources/${id}/tags`),
  setResourceTags: (id: number, tagIds: number[]): Promise<void> =>
    request("PUT", `/api/resources/${id}/tags`, { tag_ids: tagIds }),

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
  getUnitConfig: (): Promise<{ pd_hours: number; pm_workdays: number }> =>
    request("GET", "/api/config/units"),

  // ---- Settings ----
  getSettings: (): Promise<Settings> => request("GET", "/api/settings"),
  updateSettings: (settings: Settings): Promise<void> =>
    request("PUT", "/api/settings", settings),

  // ---- Phase 2: allocations ----
  createAllocation: (resourceId: number, taskId: number, start: string, end: string, percent: number): Promise<number> =>
    request("POST", "/api/allocations", { resource_id: resourceId, task_id: taskId, start, end, percent }),
  deleteAllocation: (id: number): Promise<void> =>
    request("DELETE", `/api/allocations/${id}`),
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
  listTimeOff: (): Promise<TimeOff[]> => request("GET", "/api/calendar/time-off"),
  deleteHoliday: (id: number): Promise<void> =>
    request("DELETE", `/api/calendar/holidays/${id}`),
  deleteTimeOff: (id: number): Promise<void> =>
    request("DELETE", `/api/calendar/time-off/${id}`),

  // ---- Phase 2: teams ----
  listTeams: (): Promise<Team[]> => request("GET", "/api/teams"),
  createTeam: (name: string, description: string | null): Promise<number> =>
    request("POST", "/api/teams", { name, description }),
  deleteTeam: (id: number): Promise<void> =>
    request("DELETE", `/api/teams/${id}`),
  listTeamMembers: (teamId: number): Promise<TeamMember[]> =>
    request("GET", `/api/teams/${teamId}/members`),
  addTeamMember: (teamId: number, resourceId: number, role: string | null): Promise<void> =>
    request("POST", `/api/teams/${teamId}/members`, { resource_id: resourceId, role }),
  removeTeamMember: (teamId: number, resourceId: number): Promise<void> =>
    request("DELETE", `/api/teams/${teamId}/members/${resourceId}`),
  setTeamOverride: (override: TeamOverride): Promise<void> =>
    request("PUT", "/api/teams/overrides", override),
  getTeamOverride: (teamId: number): Promise<TeamOverride | null> =>
    request("GET", `/api/teams/${teamId}/override`),

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
  listOptimizationRuns: (offset: number, limit: number): Promise<RunList> =>
    request("GET", `/api/optimization/runs?offset=${offset}&limit=${limit}`),
  getOptimizationRun: (runId: number): Promise<RunResult> =>
    request("GET", `/api/optimization/runs/${runId}`),
  applySolution: (runId: number): Promise<number> =>
    request("POST", `/api/optimization/runs/${runId}/apply`),
  rejectSolution: (runId: number): Promise<void> =>
    request("POST", `/api/optimization/runs/${runId}/reject`),

  // ---- Phase 5: reports ----
  /** Fetch a report file and trigger a browser download (no Tauri save dialog — the app is HTTP). */
  async exportReport(kind: ReportKind, projectId: number | null, start: string, end: string, format: ReportFormat): Promise<boolean> {
    const params = new URLSearchParams({ start, end, format });
    if (projectId != null) params.set("project_id", String(projectId));
    const res = await fetch(`${BASE}/api/reports/${kind}?${params}`);
    if (!res.ok) throw new Error(await res.text().catch(() => "export failed"));
    triggerDownload(await res.blob(), `${kind}.${format}`);
    return true;
  },
  async exportSnapshot(start: string, end: string): Promise<boolean> {
    const params = new URLSearchParams({ start, end });
    const res = await fetch(`${BASE}/api/reports/snapshot?${params}`);
    if (!res.ok) throw new Error(await res.text().catch(() => "export failed"));
    triggerDownload(await res.blob(), "workforce-snapshot.json");
    return true;
  },
  /** Report roadmap with available formats (design §8 / G5). */
  getReportCatalog: (): Promise<ReportCatalogEntry[]> =>
    request("GET", "/api/reports/catalog"),
};

export const reportKinds = ["ResourceUtilization", "TeamUtilization", "ProjectBurn", "AiDecisions", "Cost"] as const;
export type ReportKind = typeof reportKinds[number];
export type ReportFormat = "csv" | "xlsx" | "pdf";

/** A report catalog entry from the backend (design §8 roadmap mapping). */
export interface ReportCatalogEntry {
  kind: string;
  title: string;
  description: string;
  formats: string[];
  accepts_project_id: boolean;
  mvp: boolean;
}

/** Trigger a browser file download from a Blob (used by report exports). */
function triggerDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Defer revocation: a.click() only queues the navigation/download as a separate task,
  // so revoking synchronously can race the browser's download (fails on Firefox/Safari).
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
