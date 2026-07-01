import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { ObjectiveWeights, RunResult, RunRow } from "@/types";

import { invalidateAllocationDerivedViews } from "./invalidate";

export function useRunOptimizationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<RunResult, Error, { projectId: number; weights: ObjectiveWeights | null }>({
    mutationFn: (args) =>
      apiFetch<RunResult>(`/api/optimization/run/${args.projectId}`, {
        method: "POST",
        body: args.weights ?? undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
      // 接受方案后衍生的 allocation/workload/gantt 等也会变，applySolution 会负责失效
    },
  });
}

export function useListOptimizationRunsQuery(limit: number | null) {
  const { apiFetch } = useApiFetch();
  return useQuery<RunRow[]>({
    queryKey: ["optimization-runs", limit],
    queryFn: () =>
      apiFetch<RunRow[]>(`/api/optimization/runs${limit != null ? `?limit=${limit}` : ""}`),
  });
}

export function useApplySolutionMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, number>({
    mutationFn: (runId) => apiFetch<number>(`/api/optimization/runs/${runId}/apply`, { method: "POST" }),
    onSuccess: () => {
      // 接受方案会改 allocations → 失效所有 allocation 衍生视图
      invalidateAllocationDerivedViews(queryClient);
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
    },
  });
}

export function useRejectSolutionMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (runId) => apiFetch<void>(`/api/optimization/runs/${runId}/reject`, { method: "POST" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
    },
  });
}
