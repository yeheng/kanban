import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Resource, ResourceSkill, ResourceTag } from "@/types";

export function useListResourcesQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Resource[]>({
    queryKey: ["resources"],
    queryFn: () => apiFetch<Resource[]>("/api/resources"),
  });
}

export function useCreateResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; email: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/resources", { method: "POST", body: { name: args.name, email: args.email } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useUpdateResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number; name: string; email: string | null;
    availableFrom?: string | null; availableTo?: string | null;
    dailyCapacityPd?: number | null; dailyRatePd?: number | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/resources/${args.id}`, {
        method: "PATCH",
        body: {
          name: args.name, email: args.email,
          available_from: args.availableFrom ?? null, available_to: args.availableTo ?? null,
          daily_capacity_pd: args.dailyCapacityPd ?? null, daily_rate_pd: args.dailyRatePd ?? null,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useDeleteResourceMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/resources/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useGetResourceSkillsQuery(id: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceSkill[]>({
    queryKey: ["resource-skills", id],
    queryFn: () => apiFetch<ResourceSkill[]>(`/api/resources/${id}/skills`),
  });
}

export function useSetResourceSkillsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; skills: [number, number][] }>({
    mutationFn: (args) => apiFetch<void>(`/api/resources/${args.id}/skills`, { method: "PUT", body: { skills: args.skills } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["resource-skills", variables.id] });
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}

export function useGetResourceTagsQuery(id: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<ResourceTag[]>({
    queryKey: ["resource-tags", id],
    queryFn: () => apiFetch<ResourceTag[]>(`/api/resources/${id}/tags`),
  });
}

export function useSetResourceTagsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; tagIds: number[] }>({
    mutationFn: (args) => apiFetch<void>(`/api/resources/${args.id}/tags`, { method: "PUT", body: { tag_ids: args.tagIds } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["resource-tags", variables.id] });
      queryClient.invalidateQueries({ queryKey: ["resources"] });
    },
  });
}
