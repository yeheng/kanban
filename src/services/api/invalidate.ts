import type { QueryClient } from "@tanstack/vue-query";

/**
 * 失效所有由 allocation 写入派生的视图缓存。
 *
 * 对应旧 refresh 总线的 bump("allocations","workload","gantt","kanban","calendar")。
 * 注意：vue-query 的 invalidateQueries 用 partialMatchKey（数组元素逐位 ===），
 * 不是字符串前缀匹配——故必须用真实 queryKey 的首段（如 ["gantt-project"] 而非 ["gantt"]），
 * 否则不匹配任何已注册 query（静默 no-op）。
 */
export function invalidateAllocationDerivedViews(queryClient: QueryClient): void {
  // allocation 列表（listAllocations 用 ["allocations", projectId]）
  queryClient.invalidateQueries({ queryKey: ["allocations"] });
  // workload 各查询（workload.api.ts: resource/team/overloads/burn）
  queryClient.invalidateQueries({ queryKey: ["workload-resource"] });
  queryClient.invalidateQueries({ queryKey: ["workload-team"] });
  queryClient.invalidateQueries({ queryKey: ["workload-overloads"] });
  queryClient.invalidateQueries({ queryKey: ["workload-burn"] });
  // gantt bars（gantt.api.ts: gantt-project/gantt-resource）
  queryClient.invalidateQueries({ queryKey: ["gantt-project"] });
  queryClient.invalidateQueries({ queryKey: ["gantt-resource"] });
  // kanban 任务卡（tasks.api.ts: ["kanban", projectId]）
  queryClient.invalidateQueries({ queryKey: ["kanban"] });
  // 日历占用网格（gantt.api.ts 的 dailyOccupancy 用 ["occupancy", start, end]）
  queryClient.invalidateQueries({ queryKey: ["occupancy"] });
}

/**
 * 失效由日历写入（holiday/time-off/work-week）派生的视图缓存。
 *
 * 对应旧 refresh 总线的 bump("workload","calendar")。日历变更影响利用率计算，
 * 故须刷新 workload 与 occupancy。
 */
export function invalidateCalendarDerivedViews(queryClient: QueryClient): void {
  queryClient.invalidateQueries({ queryKey: ["workload-resource"] });
  queryClient.invalidateQueries({ queryKey: ["workload-team"] });
  queryClient.invalidateQueries({ queryKey: ["workload-overloads"] });
  queryClient.invalidateQueries({ queryKey: ["occupancy"] });
}
