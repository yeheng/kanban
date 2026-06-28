import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Skill, Tag } from "../types";

export const useCatalogStore = defineStore("catalog", () => {
  const skills = ref<Skill[]>([]);
  const tags = ref<Tag[]>([]);
  async function load() { [skills.value, tags.value] = await Promise.all([api.listSkills(), api.listTags()]); }
  async function ensureSkill(name: string) { const id = await api.ensureSkill(name); await load(); return id; }
  async function ensureTag(name: string, color: string | null) { const id = await api.ensureTag(name, color); await load(); return id; }
  return { skills, tags, load, ensureSkill, ensureTag };
});