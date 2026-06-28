import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Holiday, WeekTemplate } from "../types";

function weekFromTemplate(t: WeekTemplate | undefined): number[] {
  // On/off bits are authoritative; frac is irrelevant to the editor (full vs half handled
  // by the UI's own state). Reflect the bit as 1 (working) or 0 (off) — simplest correct read.
  if (!t) return [1, 1, 1, 1, 1, 0, 0];
  return [t.mon, t.tue, t.wed, t.thu, t.fri, t.sat, t.sun].map((b) => (b ? 1 : 0));
}

export const useCalendarStore = defineStore("calendar", () => {
  const week = ref<number[]>([1, 1, 1, 1, 1, 0, 0]); // Mon..Sun on/off
  const holidays = ref<Holiday[]>([]);

  /** Read the persisted global work week from the DB (does NOT mutate it). */
  async function loadWeek() {
    const rows = await api.listWorkWeeks();
    week.value = weekFromTemplate(rows.find((r) => r.scope === "global"));
  }
  async function setWeek(w: number[]) { week.value = w; await api.setGlobalWorkWeek(w); }
  async function loadHolidays() { holidays.value = await api.listHolidays(); }
  async function addHoliday(day: string, fraction: number, name: string | null) { await api.addHoliday(null, day, fraction, name); await loadHolidays(); }
  async function addTimeOff(resourceId: number, day: string, fraction: number, reason: string | null) { await api.addTimeOff(resourceId, day, fraction, reason); }
  return { week, holidays, loadWeek, setWeek, loadHolidays, addHoliday, addTimeOff };
});
