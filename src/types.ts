export interface Project {
  id: number; name: string; description: string | null;
  start_date: string | null; end_date: string | null;
  priority: number; budget_pd: number;
  max_parallel_tasks_per_day: number | null; status: string;
}
export interface KanbanTask {
  id: number; project_id: number; parent_task_id: number | null; title: string;
  is_long_term: number; segment_kind: string | null; status: string;
  description: string | null; sort_order: number; estimate_pd: number;
  start_date: string | null; end_date: string | null;
  assignee: string | null; skill_count: number;
}
export interface Skill { id: number; name: string; }
export interface Tag { id: number; name: string; color: string | null; }
/** Resource↔skill with resolved name + proficiency 1..5 (design §3.3.5). */
export interface ResourceSkill {
  resource_id: number; skill_id: number; skill_name: string;
  proficiency: number; evidence: string | null;
}
/** Resource↔tag with resolved name + color (design §3.3.6). */
export interface ResourceTag {
  resource_id: number; tag_id: number; tag_name: string; color: string | null;
}
export interface Resource {
  id: number; name: string; email: string | null;
  available_from: string | null; available_to: string | null;
  status: string; daily_capacity_pd: number | null;
  daily_rate_pd: number | null; max_parallel_tasks_per_day: number | null;
  metadata: string | null;
}
export type TaskStatus = "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled";

// Phase 2: workload / allocations / calendar / teams
export interface Task {
  id: number; project_id: number; parent_task_id: number | null; title: string;
  description: string | null; estimate_pd: number; start_date: string | null;
  end_date: string | null; is_long_term: number; segment_kind: string | null;
  status: string; sort_order: number;
}
/** Server-computed utilization band (per-team effective thresholds, design §3.3.8a). */
export type UtilBand = "under" | "green" | "yellow" | "red";
export interface ResourceSummary {
  resource_id: number; capacity_pd: number; workload_pd: number;
  utilization: number; overloaded: boolean; status: UtilBand;
}
export interface TeamSummary {
  team_id: number; capacity_pd: number; workload_pd: number;
  utilization: number; overloaded_members: number[]; status: UtilBand;
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
export interface TeamOverride {
  team_id: number;
  pd_hours: number | null;
  pm_workdays: number | null;
  overload_threshold: number | null;
  underload_threshold: number | null;
  utilization_green: number | null;
  utilization_yellow: number | null;
}
export interface TimeOff { id: number; resource_id: number; day: string; fraction: number; reason: string | null; }
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
  workload_pd: number; capacity_pd: number; utilization: number; status: UtilBand;
}

// Phase 4: AI optimization
export interface ObjectiveWeights { skill_fit: number; balance: number; budget: number; }
export interface ScoredAssignment {
  resource_id: number; task_id: number;
  resource_name: string; task_title: string;
  start: string; end: string;
  percent: number; score: number; rationale: string;
}
export interface SolutionMetrics { overall: number; skill_fit: number; scheduled_ratio: number; fairness: number; }
export interface Solution { run_id: number; assignments: ScoredAssignment[]; unscheduled: number[]; metrics: SolutionMetrics; }
export interface RunResult { run_id: number; plan: { solution: Solution; explanation_md: string; }; }
export interface RunRow { id: number; objective: string; status: string; applied: number; score_overall: number | null; created_at: string; }
export interface RunList { rows: RunRow[]; total: number; }

export type Suggestion =
  | { kind: "swap_resource"; task_id: number; new_resource_id: number }
  | { kind: "change_percent"; task_id: number; new_percent: number }
  | { kind: "widen_window"; task_id: number; new_start: string; new_end: string }
  | { kind: "drop_dependency"; task_id: number; predecessor_id: number }
  | { kind: "add_resource"; resource_id: number }
  | { kind: "widen_resource_window"; resource_id: number; new_available_from: string; new_available_to: string }
  | { kind: "change_resource_capacity"; resource_id: number; new_daily_capacity_pd: number }
  | { kind: "upsert_resource_skill"; resource_id: number; skill_id: number; new_proficiency: number };

export interface SuggestionItem {
  id: number;
  suggestion: Suggestion;
  rationale_md: string;
  status: "proposed" | "accepted" | "skipped" | "applied";
}

// Global settings (design §3.3.1)
export interface Settings {
  default_unit: "PD" | "PM";
  pd_hours: number;
  pm_workdays: number;
  ai_provider: string;
  ai_base_url: string | null;
  ai_api_key_enc: string | null;
  secret_store: "keychain" | "encrypted_file";
  ai_chat_model: string;
  embed_provider: string;
  embed_base_url: string | null;
  embed_api_key_enc: string | null;
  embed_model: string;
  embed_dim: number;
  solver_backend: "good_lp" | "greedy" | "hungarian";
  solver_timeout_ms: number;
  locale: string;
  use_semantic_scorer: boolean;
  use_llm_explainer: boolean;
  ai_explanation_prompt: string;
  ai_explanation_preamble: string;
  overload_threshold: number;
  underload_threshold: number;
  utilization_green: number;
  utilization_yellow: number;
}
