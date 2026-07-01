import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { KanbanTask, Task, TaskStatus } from "@/types";

/** [skillId, proficiency, required, weight] — 与旧 api/index.ts 的 SkillReq 一致。 */
export type SkillReq = [number, number, boolean, number];

export function useListTasksQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<Task[]>({
    queryKey: ["tasks", projectId],
    queryFn: () => apiFetch<Task[]>(`/api/projects/${projectId}/tasks`),
  });
}

export function useKanbanTasksQuery(projectId: number) {
  const { apiFetch } = useApiFetch();
  return useQuery<KanbanTask[]>({
    queryKey: ["kanban", projectId],
    queryFn: () => apiFetch<KanbanTask[]>(`/api/projects/${projectId}/kanban`),
  });
}

export function useCreateTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/tasks", {
        method: "POST",
        body: {
          project_id: args.projectId,
          title: args.title,
          estimate_pd: args.estimatePd,
          start: args.start,
          end: args.end,
          skill_reqs: args.skillReqs,
          tag_ids: args.tagIds,
          description: args.description ?? null,
          is_long_term: args.isLongTerm ?? false,
          parent_task_id: args.parentTaskId ?? null,
          segment_kind: args.segmentKind ?? null,
          sort_order: 0,
        },
      }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
      queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
    },
  });
}

export function useUpdateTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number; projectId?: number;
    title: string; estimatePd: number;
    start: string | null; end: string | null;
    description?: string | null;
    isLongTerm?: boolean; parentTaskId?: number | null; segmentKind?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.id}`, {
        method: "PATCH",
        body: {
          title: args.title,
          description: args.description ?? null,
          estimate_pd: args.estimatePd,
          start: args.start,
          end: args.end,
          is_long_term: args.isLongTerm ?? false,
          parent_task_id: args.parentTaskId ?? null,
          segment_kind: args.segmentKind ?? null,
        },
      }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useDeleteTaskMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; projectId?: number }>({
    mutationFn: (args) => apiFetch<void>(`/api/tasks/${args.id}`, { method: "DELETE" }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useSetTaskStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: TaskStatus; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.id}/status`, { method: "PATCH", body: { status: args.status } }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["tasks", variables.projectId] });
        queryClient.invalidateQueries({ queryKey: ["kanban", variables.projectId] });
      }
    },
  });
}

export function useAddDependencyMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { taskId: number; predecessorId: number; lagDays?: number; projectId?: number }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/tasks/${args.taskId}/dependencies`, {
        method: "POST",
        body: { predecessor_id: args.predecessorId, lag_days: args.lagDays ?? 0, dep_type: "finish_to_start" },
      }),
    onSuccess: (_data, variables) => {
      if (variables.projectId != null) {
        queryClient.invalidateQueries({ queryKey: ["dependencies", variables.projectId] });
      }
    },
  });
}
