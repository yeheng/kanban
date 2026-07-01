import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Holiday, TimeOff, WeekTemplate } from "@/types";

import { invalidateCalendarDerivedViews } from "./invalidate";

export function useListWorkWeeksQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<WeekTemplate[]>({
    queryKey: ["work-weeks"],
    queryFn: () => apiFetch<WeekTemplate[]>("/api/calendar/work-week"),
  });
}

export function useSetGlobalWorkWeekMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number[]>({
    mutationFn: (week) => apiFetch<void>("/api/calendar/work-week", { method: "POST", body: { week } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["work-weeks"] });
      // work-week 变更影响利用率与占用网格
      invalidateCalendarDerivedViews(queryClient);
    },
  });
}

export function useListHolidaysQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Holiday[]>({
    queryKey: ["holidays"],
    queryFn: () => apiFetch<Holiday[]>("/api/calendar/holidays"),
  });
}

export function useAddHolidayMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { projectId: number | null; day: string; fraction: number | null; name: string | null }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/calendar/holidays", {
        method: "POST",
        body: { project_id: args.projectId, day: args.day, fraction: args.fraction, name: args.name },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["holidays"] });
      invalidateCalendarDerivedViews(queryClient);
    },
  });
}

export function useDeleteHolidayMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/calendar/holidays/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["holidays"] });
      invalidateCalendarDerivedViews(queryClient);
    },
  });
}

export function useListTimeOffQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<TimeOff[]>({
    queryKey: ["time-off"],
    queryFn: () => apiFetch<TimeOff[]>("/api/calendar/time-off"),
  });
}

export function useAddTimeOffMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { resourceId: number; day: string; fraction: number | null; reason: string | null }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/calendar/time-off", {
        method: "POST",
        body: { resource_id: args.resourceId, day: args.day, fraction: args.fraction, reason: args.reason },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-off"] });
      invalidateCalendarDerivedViews(queryClient);
    },
  });
}

export function useDeleteTimeOffMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/calendar/time-off/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["time-off"] });
      invalidateCalendarDerivedViews(queryClient);
    },
  });
}
