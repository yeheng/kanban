import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { api } from "../api";

export type DisplayUnit = "PD" | "PM";

const STORAGE_KEY = "dev-resource-kanban.unit";

function initialUnit(): DisplayUnit {
  const saved = globalThis.localStorage?.getItem(STORAGE_KEY);
  return saved === "PM" ? "PM" : "PD";
}

export const useUnitStore = defineStore("unit", () => {
  const unit = ref<DisplayUnit>(initialUnit());
  // Effective PD/PM denominator. Defaults to 20 (design §2.9); replaced with the backend
  // global setting on load, and optionally with a team-level `pm_workdays` override when a
  // team context is active (design §3.3.8a).
  const pdPerPm = ref(20);

  const options = computed(() => [
    { label: "PD", value: "PD" },
    { label: "PM", value: "PM" },
  ]);

  watch(unit, (value) => {
    globalThis.localStorage?.setItem(STORAGE_KEY, value);
  });

  /// Load the global PD/PM constants from the backend (`GET /api/config/units`).
  /// Call once on app start; falls back to 20 if the request fails (offline-first).
  async function loadGlobal() {
    try {
      const cfg = await api.getUnitConfig();
      pdPerPm.value = cfg.pm_workdays || 20;
    } catch {
      pdPerPm.value = 20;
    }
  }

  /// Apply a team-level `pm_workdays` override (design §3.3.8a). Pass null to revert to
  /// the global value. Views that know the active team call this on team change.
  function applyTeamOverride(pmWorkdays: number | null) {
    pdPerPm.value = pmWorkdays ?? globalPmWorkdays.value;
  }

  // Cached global value so a team override can be reverted without a refetch.
  const globalPmWorkdays = ref(20);
  async function loadGlobalAndCache() {
    await loadGlobal();
    globalPmWorkdays.value = pdPerPm.value;
  }

  function formatPd(value: number, digits = 1): string {
    if (unit.value === "PM") {
      return `${(value / pdPerPm.value).toFixed(digits)} PM`;
    }
    return `${value.toFixed(digits)} PD`;
  }

  return { unit, pdPerPm, options, formatPd, loadGlobal: loadGlobalAndCache, applyTeamOverride };
});
