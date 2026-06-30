import { defineStore } from "pinia";
import { reactive } from "vue";

/**
 * Shared cross-view refresh/invalidation bus (design G4: "任一分配变更后，三视图在 ≤500ms
 * 内完成重渲染").
 *
 * Each scope holds a monotonically-increasing `version`. Stores/views that cache data for a
 * scope `watch` its version and reload when it bumps. Mutating stores (allocations, tasks,
 * calendar, optimization-accept) call `bump(scope)` for every scope their write invalidates,
 * instead of each view guessing or relying on lazy mount-time reloads.
 *
 * Scopes:
 *  - allocations: allocation list (AllocationsView)
 *  - workload:    resource/team summaries, overloads, project burn (Dashboard)
 *  - gantt:       gantt bars + dependency edges (GanttView)
 *  - kanban:      kanban task cards (KanbanView)
 *  - calendar:    calendar occupancy grid (CalendarView / CalendarGridView)
 *  - resources:   resource list incl. skills/tags (ResourcesView)
 */
export type RefreshScope =
  | "allocations"
  | "workload"
  | "gantt"
  | "kanban"
  | "calendar"
  | "resources";

export const useRefreshStore = defineStore("refresh", () => {
  const version = reactive<Record<RefreshScope, number>>({
    allocations: 0,
    workload: 0,
    gantt: 0,
    kanban: 0,
    calendar: 0,
    resources: 0,
  });

  /// Bump one or more scopes, signalling that cached data for them is now stale and views
  /// should reload. Returns the bumped scopes for test assertions.
  function bump(...scopes: RefreshScope[]): RefreshScope[] {
    for (const s of scopes) version[s] += 1;
    return scopes;
  }

  return { version, bump };
});
