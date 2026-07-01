import { useQuery } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { ProjectBurn, ResourceSummary, TeamSummary } from "@/types";

export function useResourceSummaryQuery(resourceId: number, start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSummary>({
    queryKey: ["workload-resource", resourceId, start, end],
    queryFn: () =>
      apiFetch<ResourceSummary>(
        `/api/workload/resources/${resourceId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useTeamSummaryQuery(teamId: number, start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<TeamSummary>({
    queryKey: ["workload-team", teamId, start, end],
    queryFn: () =>
      apiFetch<TeamSummary>(
        `/api/workload/teams/${teamId}?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useOverloadsQuery(start: string, end: string) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSummary[]>({
    queryKey: ["workload-overloads", start, end],
    queryFn: () =>
      apiFetch<ResourceSummary[]>(
        `/api/workload/overloads?start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`,
      ),
  });
}

export function useProjectBurnQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ProjectBurn>({
    queryKey: ["workload-burn", projectId],
    queryFn: () => apiFetch<ProjectBurn>(`/api/projects/${projectId}/burn`),
  });
}
