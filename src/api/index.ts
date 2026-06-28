import { invoke } from "@tauri-apps/api/core";
import type { Project, KanbanTask, Skill, Tag, Resource, TaskStatus } from "../types";

export type SkillReq = [number, number, boolean, number];

export const api = {
  listProjects: (): Promise<Project[]> => invoke("list_projects"),
  createProject: (name: string, priority: number, budgetPd: number): Promise<number> =>
    invoke("create_project", { name, priority, budgetPd }),

  listSkills: (): Promise<Skill[]> => invoke("list_skills"),
  ensureSkill: (name: string): Promise<number> => invoke("ensure_skill", { name }),
  listTags: (): Promise<Tag[]> => invoke("list_tags"),
  ensureTag: (name: string, color: string | null): Promise<number> => invoke("ensure_tag", { name, color }),

  createTask: (args: {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
  }): Promise<number> =>
    invoke("create_task", {
      projectId: args.projectId, title: args.title, estimatePd: args.estimatePd,
      start: args.start, end: args.end, skillReqs: args.skillReqs, tagIds: args.tagIds,
      description: null, isLongTerm: false, sortOrder: 0,
    }),
  setTaskStatus: (id: number, status: TaskStatus): Promise<void> =>
    invoke("set_task_status", { id, status }),
  kanbanTasks: (projectId: number): Promise<KanbanTask[]> => invoke("kanban_tasks", { projectId }),

  listResources: (): Promise<Resource[]> => invoke("list_resources"),
  createResource: (name: string, email: string | null): Promise<number> =>
    invoke("create_resource", { name, email }),
};