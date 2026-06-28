import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Team, TeamMember } from "../types";

export const useTeamsStore = defineStore("teams", () => {
  const items = ref<Team[]>([]);
  const members = ref<TeamMember[]>([]);
  async function load() { items.value = await api.listTeams(); }
  async function loadMembers(teamId: number) { members.value = await api.listTeamMembers(teamId); }
  return { items, members, load, loadMembers };
});
