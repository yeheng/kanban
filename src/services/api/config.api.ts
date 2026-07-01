import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Settings, Thresholds } from "@/types";

export function useGetThresholdsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Thresholds>({
    queryKey: ["thresholds"],
    queryFn: () => apiFetch<Thresholds>("/api/thresholds"),
  });
}

export function useGetUnitConfigQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<{ pd_hours: number; pm_workdays: number }>({
    queryKey: ["unit-config"],
    queryFn: () => apiFetch<{ pd_hours: number; pm_workdays: number }>("/api/config/units"),
  });
}

export function useGetSettingsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Settings>({
    queryKey: ["settings"],
    queryFn: () => apiFetch<Settings>("/api/settings"),
  });
}

export function useUpdateSettingsMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, Settings>({
    mutationFn: (settings) => apiFetch<void>("/api/settings", { method: "PUT", body: settings }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}
