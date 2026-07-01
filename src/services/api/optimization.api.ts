import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { type MaybeRef, computed, toValue } from "vue";
import { useApiFetch } from "../fetch";
import type { ObjectiveWeights, RunList, RunResult, RunRow, SuggestionItem } from "@/types";

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

export function useListOptimizationRunsQuery(
  offset: MaybeRef<number>,
  limit: MaybeRef<number>,
) {
  const { apiFetch } = useApiFetch();
  const o = computed(() => toValue(offset));
  const l = computed(() => toValue(limit));
  return useQuery<RunList>({
    queryKey: computed(() => ["optimization-runs", o.value, l.value]),
    queryFn: () =>
      apiFetch<RunList>(`/api/optimization/runs?offset=${o.value}&limit=${l.value}`),
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

export function useGetOptimizationRunQuery(runId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(runId));
  return useQuery<RunResult>({
    queryKey: computed(() => ["optimization-run", id.value]),
    queryFn: () => apiFetch<RunResult>(`/api/optimization/runs/${id.value}`),
    enabled: () => id.value != null,
  });
}

export function useListSuggestionsQuery(runId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(runId));
  return useQuery<SuggestionItem[]>({
    queryKey: computed(() => ["optimization-suggestions", id.value]),
    queryFn: () => apiFetch<SuggestionItem[]>(`/api/optimization/runs/${id.value}/suggestions`),
    enabled: () => id.value != null,
  });
}

export function useRerunMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<RunResult, Error, { runId: number; suggestionIds: number[] }>({
    mutationFn: (args) =>
      apiFetch<RunResult>(`/api/optimization/runs/${args.runId}/rerun`, {
        method: "POST",
        body: { suggestion_ids: args.suggestionIds },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["optimization-runs"] });
    },
  });
}

export function useSetSuggestionStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: "accepted" | "skipped" }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/optimization/suggestions/${args.id}`, {
        method: "PATCH",
        body: { status: args.status },
      }),
    onSuccess: (_, args) => {
      queryClient.invalidateQueries({ queryKey: ["optimization-suggestions", args.id] });
    },
  });
}
