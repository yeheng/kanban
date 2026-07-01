import { defineStore } from "pinia";
import { ref } from "vue";

export const useProjectsStore = defineStore("projects", () => {
  const current = ref<number | null>(null);

  function select(id: number) {
    current.value = id;
  }

  return { current, select };
});
