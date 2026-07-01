import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Settings } from "../types";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings | null>(null);
  const loading = ref(false);
  const saving = ref(false);

  async function load() {
    loading.value = true;
    try {
      settings.value = await api.getSettings();
    } finally {
      loading.value = false;
    }
  }

  async function save(draft: Settings) {
    saving.value = true;
    try {
      await api.updateSettings(draft);
      settings.value = { ...draft };
    } finally {
      saving.value = false;
    }
  }

  return { settings, loading, saving, load, save };
});
