import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Resource } from "../types";

export const useResourcesStore = defineStore("resources", () => {
  const items = ref<Resource[]>([]);
  async function load() { items.value = await api.listResources(); }
  async function create(name: string, email: string | null) { await api.createResource(name, email); await load(); }
  return { items, load, create };
});