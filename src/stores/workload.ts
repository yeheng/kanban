import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { ResourceSummary, TeamSummary, ProjectBurn, Thresholds } from "../types";

export const useWorkloadStore = defineStore("workload", () => {
  const resourceSummaries = ref<ResourceSummary[]>([]);
  const overloads = ref<ResourceSummary[]>([]);
  const thresholds = ref<Thresholds>({ overload: 1.1, underload: 0.5, green: 0.7, yellow: 1.0 });
  const teamSummary = ref<TeamSummary | null>(null);
  const projectBurn = ref<ProjectBurn | null>(null);

  async function loadThresholds() { thresholds.value = await api.getThresholds(); }
  async function loadResourceSummaries(resourceIds: number[], start: string, end: string) {
    resourceSummaries.value = await Promise.all(resourceIds.map((id) => api.resourceSummary(id, start, end)));
  }
  async function loadOverloads(start: string, end: string) { overloads.value = await api.overloads(start, end); }
  async function loadTeamSummary(teamId: number, start: string, end: string) { teamSummary.value = await api.teamSummary(teamId, start, end); }
  async function loadProjectBurn(projectId: number) { projectBurn.value = await api.projectBurn(projectId); }

  /** Color band for a utilization value using global thresholds. */
  function band(util: number): "under" | "green" | "yellow" | "red" {
    const t = thresholds.value;
    if (util >= t.overload) return "red";
    if (util >= t.yellow) return "yellow";
    if (util >= t.green) return "green";
    return "under";
  }
  return { resourceSummaries, overloads, thresholds, teamSummary, projectBurn,
           loadThresholds, loadResourceSummaries, loadOverloads, loadTeamSummary, loadProjectBurn, band };
});
