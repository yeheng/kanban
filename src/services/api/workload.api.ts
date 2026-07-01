import { type MaybeRef, computed, toValue } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { ProjectBurn, ResourceSummary, TeamSummary } from "@/types";

export function useResourceSummaryQuery(
  resourceId: MaybeRef<number | null>,
  start: MaybeRef<string>,
  end: MaybeRef<string>,
) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(resourceId));
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<ResourceSummary>({
    queryKey: computed(() => ["workload-resource", id.value, s.value, e.value]),
    queryFn: () =>
      apiFetch<ResourceSummary>(
        `/api/workload/resources/${id.value}?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => id.value != null && !!s.value && !!e.value,
  });
}

export function useTeamSummaryQuery(
  teamId: MaybeRef<number | null>,
  start: MaybeRef<string>,
  end: MaybeRef<string>,
) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<TeamSummary>({
    queryKey: computed(() => ["workload-team", id.value, s.value, e.value]),
    queryFn: () =>
      apiFetch<TeamSummary>(
        `/api/workload/teams/${id.value}?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => id.value != null && !!s.value && !!e.value,
  });
}

export function useOverloadsQuery(start: MaybeRef<string>, end: MaybeRef<string>) {
  const { apiFetch } = useApiFetch();
  const s = computed(() => toValue(start));
  const e = computed(() => toValue(end));
  return useQuery<ResourceSummary[]>({
    queryKey: computed(() => ["workload-overloads", s.value, e.value]),
    queryFn: () =>
      apiFetch<ResourceSummary[]>(
        `/api/workload/overloads?start=${encodeURIComponent(s.value)}&end=${encodeURIComponent(e.value)}`,
      ),
    enabled: () => !!s.value && !!e.value,
  });
}

export function useProjectBurnQuery(projectId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(projectId));
  return useQuery<ProjectBurn>({
    queryKey: computed(() => ["workload-burn", id.value]),
    queryFn: () => apiFetch<ProjectBurn>(`/api/projects/${id.value}/burn`),
    enabled: () => id.value != null,
  });
}
