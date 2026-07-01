import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { DayOccupancy, DepEdge, GanttBar } from "@/types";

export function useGanttProjectQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<GanttBar[]>({
    queryKey: ["gantt-project", projectId],
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/projects/${projectId}`),
  });
}

export function useGanttResourceQuery(resourceId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<GanttBar[]>({
    queryKey: ["gantt-resource", resourceId],
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/resources/${resourceId}`),
  });
}

export function useDependenciesForProjectQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<DepEdge[]>({
    queryKey: ["dependencies", projectId],
    queryFn: () => apiFetch<DepEdge[]>(`/api/projects/${projectId}/dependencies`),
  });
}

export function useDailyOccupancyQuery(start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<DayOccupancy[]>({
    queryKey: ["occupancy", start, end],
    queryFn: () =>
      apiFetch<DayOccupancy[]>(`/api/occupancy?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`),
  });
}
