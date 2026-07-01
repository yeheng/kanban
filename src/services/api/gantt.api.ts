import { type MaybeRef, computed, toValue } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { DayOccupancy, DepEdge, GanttBar } from "@/types";

export function useGanttProjectQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<GanttBar[]>({
    queryKey: computed(() => ["gantt-project", id.value]),
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/projects/${id.value}`),
    enabled: () => id.value != null,
  });
}

export function useGanttResourceQuery(resourceId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(resourceId));
  return useQuery<GanttBar[]>({
    queryKey: computed(() => ["gantt-resource", id.value]),
    queryFn: () => apiFetch<GanttBar[]>(`/api/gantt/resources/${id.value}`),
    enabled: () => id.value != null,
  });
}

export function useDependenciesForProjectQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<DepEdge[]>({
    queryKey: computed(() => ["dependencies", id.value]),
    queryFn: () => apiFetch<DepEdge[]>(`/api/projects/${id.value}/dependencies`),
    enabled: () => id.value != null,
  });
}

export function useDailyOccupancyQuery(start: MaybeRef<string>, end: MaybeRef<string>) {
  const { apiFetch } = useApiFetch();
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<DayOccupancy[]>({
    queryKey: computed(() => ["occupancy", s.value, e.value]),
    queryFn: () =>
      apiFetch<DayOccupancy[]>(`/api/occupancy?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`),
    enabled: () => !!s.value && !!e.value,
  });
}
