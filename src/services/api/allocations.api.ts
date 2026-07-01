import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { AllocationView } from "@/types";

import { invalidateAllocationDerivedViews } from "./invalidate";

export function useListAllocationsQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<AllocationView[]>({
    queryKey: ["allocations", projectId],
    queryFn: () => apiFetch<AllocationView[]>(`/api/projects/${projectId}/allocations`),
  });
}

export function useCreateAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, {
    resourceId: number; taskId: number; start: string; end: string; percent: number; projectId?: number;
  }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/allocations", {
        method: "POST",
        body: { resource_id: args.resourceId, task_id: args.taskId, start: args.start, end: args.end, percent: args.percent },
      }),
    onSuccess: () => {
      // allocation 写入失效所有 allocation 衍生视图（对应旧 refresh bump 的多 scope）
      invalidateAllocationDerivedViews(queryClient);
    },
  });
}

export function useUpdateAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; start: string; end: string; percent: number; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/allocations/${args.id}`, { method: "PUT", body: { start: args.start, end: args.end, percent: args.percent } }),
    onSuccess: () => {
      invalidateAllocationDerivedViews(queryClient);
    },
  });
}

export function useDeleteAllocationMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; projectId?: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/allocations/${args.id}`, { method: "DELETE" }),
    onSuccess: () => {
      invalidateAllocationDerivedViews(queryClient);
    },
  });
}
