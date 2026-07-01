import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Skill, Tag } from "@/types";

export function useListSkillsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Skill[]>({
    queryKey: ["skills"],
    queryFn: () => apiFetch<Skill[]>("/api/skills"),
  });
}

export function useEnsureSkillMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, string>({
    mutationFn: (name) => apiFetch<number>("/api/skills", { method: "POST", body: { name } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["skills"] });
    },
  });
}

export function useListTagsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Tag[]>({
    queryKey: ["tags"],
    queryFn: () => apiFetch<Tag[]>("/api/tags"),
  });
}

export function useEnsureTagMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; color: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/tags", { method: "POST", body: { name: args.name, color: args.color } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });
}
