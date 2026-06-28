import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Project } from "../types";

export const useProjectsStore = defineStore("projects", () => {
  const items = ref<Project[]>([]);
  const current = ref<number | null>(null);

  async function load() { items.value = await api.listProjects(); if (!current.value && items.value.length) current.value = items.value[0].id; }
  async function create(name: string, priority: number, budgetPd: number) { await api.createProject(name, priority, budgetPd); await load(); }
  function select(id: number) { current.value = id; }

  return { items, current, load, create, select };
});