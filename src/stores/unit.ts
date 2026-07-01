import { defineStore } from "pinia";
import { ref, watch } from "vue";

const STORAGE_KEY = "dev-resource-kanban.unit";

function initialUnit(): "PD" | "PM" {
  const saved = globalThis.localStorage?.getItem(STORAGE_KEY);
  return saved === "PM" ? "PM" : "PD";
}

export const useUnitStore = defineStore("unit", () => {
  const unit = ref<"PD" | "PM">(initialUnit());

  watch(unit, (value) => {
    globalThis.localStorage?.setItem(STORAGE_KEY, value);
  });

  function formatPd(pd: number | null | undefined): string {
    if (pd == null) return "—";
    return unit.value === "PM" ? `${(pd / 20).toFixed(1)} PM` : `${pd.toFixed(1)} PD`;
  }

  function applyTeamOverride(pmWorkdays: number | null): number {
    return pmWorkdays ?? 20;
  }

  return { unit, formatPd, applyTeamOverride };
});
