export interface Project {
  id: number; name: string; description: string | null;
  start_date: string | null; end_date: string | null;
  priority: number; budget_pd: number;
  max_parallel_tasks_per_day: number | null; status: string;
}
export interface KanbanTask {
  id: number; project_id: number; title: string; status: string;
  sort_order: number; estimate_pd: number;
  assignee: string | null; skill_count: number;
}
export interface Skill { id: number; name: string; }
export interface Tag { id: number; name: string; color: string | null; }
export interface Resource { id: number; name: string; email: string | null; status: string; }
export type TaskStatus = "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled";