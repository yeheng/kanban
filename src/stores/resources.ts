import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Resource, ResourceSkill, ResourceTag } from "../types";

export const useResourcesStore = defineStore("resources", () => {
  const items = ref<Resource[]>([]);
  async function load() { items.value = await api.listResources(); }
  async function create(name: string, email: string | null) { await api.createResource(name, email); await load(); }
  async function update(id: number, args: {
    name: string; email: string | null;
    availableFrom?: string | null; availableTo?: string | null;
    dailyCapacityPd?: number | null; dailyRatePd?: number | null;
  }) { await api.updateResource(id, args); await load(); }
  async function remove(id: number) { await api.deleteResource(id); await load(); }
  async function loadSkills(id: number): Promise<ResourceSkill[]> { return api.getResourceSkills(id); }
  async function saveSkills(id: number, skills: [number, number][]) { await api.setResourceSkills(id, skills); }
  async function loadTags(id: number): Promise<ResourceTag[]> { return api.getResourceTags(id); }
  async function saveTags(id: number, tagIds: number[]) { await api.setResourceTags(id, tagIds); }
  return { items, load, create, update, remove, loadSkills, saveSkills, loadTags, saveTags };
});
