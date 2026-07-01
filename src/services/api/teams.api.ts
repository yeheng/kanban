import { type MaybeRef, computed, toValue } from "vue";
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Team, TeamMember, TeamOverride } from "@/types";

export function useListTeamsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Team[]>({
    queryKey: ["teams"],
    queryFn: () => apiFetch<Team[]>("/api/teams"),
  });
}

export function useCreateTeamMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; description: string | null }>({
    mutationFn: (args) => apiFetch<number>("/api/teams", { method: "POST", body: { name: args.name, description: args.description } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
}

export function useDeleteTeamMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/teams/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
}

export function useListTeamMembersQuery(teamId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  return useQuery<TeamMember[]>({
    queryKey: computed(() => ["team-members", id.value]),
    queryFn: () => apiFetch<TeamMember[]>(`/api/teams/${id.value}/members`),
    enabled: () => id.value != null,
  });
}

export function useAddTeamMemberMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { teamId: number; resourceId: number; role: string | null }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/teams/${args.teamId}/members`, { method: "POST", body: { resource_id: args.resourceId, role: args.role } }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["team-members", variables.teamId] });
    },
  });
}

export function useRemoveTeamMemberMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { teamId: number; resourceId: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/teams/${args.teamId}/members/${args.resourceId}`, { method: "DELETE" }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["team-members", variables.teamId] });
    },
  });
}

export function useSetTeamOverrideMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, TeamOverride>({
    mutationFn: (override) => apiFetch<void>("/api/teams/overrides", { method: "PUT", body: override }),
    onSuccess: (_data, variables) => {
      // 针对该 team 的 override 缓存；override 改阈值/单位会影响该 team 的 workload 汇总
      queryClient.invalidateQueries({ queryKey: ["team-override", variables.team_id] });
      queryClient.invalidateQueries({ queryKey: ["workload-team"] });
    },
  });
}

export function useGetTeamOverrideQuery(teamId: MaybeRef<number | null>) {
  const { apiFetch } = useApiFetch();
  const id = computed(() => toValue(teamId));
  return useQuery<TeamOverride | null>({
    queryKey: computed(() => ["team-override", id.value]),
    queryFn: () => apiFetch<TeamOverride | null>(`/api/teams/${id.value}/override`),
    enabled: () => id.value != null,
  });
}
