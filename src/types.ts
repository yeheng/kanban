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

// Phase 2: workload / allocations / calendar / teams
export interface Task {
  id: number; project_id: number; parent_task_id: number | null; title: string;
  description: string | null; estimate_pd: number; start_date: string | null;
  end_date: string | null; is_long_term: number; segment_kind: string | null;
  status: string; sort_order: number;
}
export interface ResourceSummary {
  resource_id: number; capacity_pd: number; workload_pd: number;
  utilization: number; overloaded: boolean;
}
export interface TeamSummary {
  team_id: number; capacity_pd: number; workload_pd: number;
  utilization: number; overloaded_members: number[];
}
export interface ProjectBurn { project_id: number; budget_pd: number; allocated_pd: number; usage: number; }
export interface Thresholds { overload: number; underload: number; green: number; yellow: number; }
export interface AllocationView {
  id: number; resource_id: number; resource_name: string; task_id: number;
  task_title: string; project_id: number; start_date: string; end_date: string;
  percent: number; status: string; source: string;
}
export interface Team { id: number; name: string; description: string | null; }
export interface TeamMember { team_id: number; resource_id: number; role: string | null; }
export interface Holiday { id: number; project_id: number | null; day: string; fraction: number; name: string | null; }
export interface WeekTemplate {
  id: number; scope: string; project_id: number | null;
  mon: number; tue: number; wed: number; thu: number; fri: number; sat: number; sun: number;
  mon_frac: number; tue_frac: number; wed_frac: number; thu_frac: number;
  fri_frac: number; sat_frac: number; sun_frac: number;
}

// Phase 3: Gantt + calendar occupancy
export interface GanttBar {
  allocation_id: number; resource_id: number; resource_name: string;
  task_id: number; task_title: string; project_id: number; project_name: string;
  start_date: string; end_date: string; percent: number; status: string; source: string;
}
export interface DepEdge { task_id: number; predecessor_id: number; lag_days: number; dep_type: string; }
export interface DayOccupancy {
  date: string; resource_id: number; resource_name: string;
  workload_pd: number; capacity_pd: number; utilization: number;
}