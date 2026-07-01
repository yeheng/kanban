import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Project } from "@/types";

export function useListProjectsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Project[]>({
    queryKey: ["projects"],
    queryFn: () => apiFetch<Project[]>("/api/projects"),
  });
}

export function useCreateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; priority: number; budgetPd: number }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/projects", {
        method: "POST",
        body: { name: args.name, priority: args.priority, budget_pd: args.budgetPd },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useUpdateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number;
    name: string; priority: number; budgetPd: number;
    description?: string | null; start?: string | null; end?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/projects/${args.id}`, {
        method: "PATCH",
        body: {
          name: args.name, description: args.description ?? null,
          start: args.start ?? null, end: args.end ?? null,
          priority: args.priority, budget_pd: args.budgetPd,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useSetProjectStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: string }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/projects/${args.id}/status`, { method: "PATCH", body: { status: args.status } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useDeleteProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/projects/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}
