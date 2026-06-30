import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";

export type DisplayUnit = "PD" | "PM";

const STORAGE_KEY = "dev-resource-kanban.unit";

function initialUnit(): DisplayUnit {
  const saved = globalThis.localStorage?.getItem(STORAGE_KEY);
  return saved === "PM" ? "PM" : "PD";
}

export const useUnitStore = defineStore("unit", () => {
  const unit = ref<DisplayUnit>(initialUnit());
  const pdPerPm = ref(20);

  const options = computed(() => [
    { label: "PD", value: "PD" },
    { label: "PM", value: "PM" },
  ]);

  watch(unit, (value) => {
    globalThis.localStorage?.setItem(STORAGE_KEY, value);
  });

  function formatPd(value: number, digits = 1): string {
    if (unit.value === "PM") {
      return `${(value / pdPerPm.value).toFixed(digits)} PM`;
    }
    return `${value.toFixed(digits)} PD`;
  }

  return { unit, pdPerPm, options, formatPd };
});
