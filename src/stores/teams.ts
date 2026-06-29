import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Team, TeamMember, TeamOverride } from "../types";

export const useTeamsStore = defineStore("teams", () => {
  const items = ref<Team[]>([]);
  const members = ref<TeamMember[]>([]);
  async function load() { items.value = await api.listTeams(); }
  async function create(name: string, description: string | null) { await api.createTeam(name, description); await load(); }
  async function remove(id: number) { await api.deleteTeam(id); await load(); }
  async function loadMembers(teamId: number) { members.value = await api.listTeamMembers(teamId); }
  async function addMember(teamId: number, resourceId: number, role: string | null) {
    await api.addTeamMember(teamId, resourceId, role);
    await loadMembers(teamId);
  }
  async function setOverride(override: TeamOverride) {
    await api.setTeamOverride(override);
  }
  return { items, members, load, create, remove, loadMembers, addMember, setOverride };
});
