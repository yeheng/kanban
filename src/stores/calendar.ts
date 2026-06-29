import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Holiday, WeekTemplate } from "../types";

function weekFromTemplate(t: WeekTemplate | undefined): number[] {
  if (!t) return [1, 1, 1, 1, 1, 0, 0];
  // Mirror the backend's frac_of semantics (crates/db/src/repo/calendar.rs): the on/off bit
  // is authoritative; an off-day yields 0 regardless of frac, a working day yields its frac.
  // Reading the frac (not just the bit) is what makes half-days round-trip correctly.
  const f = (bit: number, frac: number) => (bit ? frac : 0);
  return [f(t.mon, t.mon_frac), f(t.tue, t.tue_frac), f(t.wed, t.wed_frac),
          f(t.thu, t.thu_frac), f(t.fri, t.fri_frac), f(t.sat, t.sat_frac), f(t.sun, t.sun_frac)];
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
  async function removeHoliday(id: number) { await api.deleteHoliday(id); await loadHolidays(); }
  async function addTimeOff(resourceId: number, day: string, fraction: number, reason: string | null) { await api.addTimeOff(resourceId, day, fraction, reason); }
  async function removeTimeOff(id: number) { await api.deleteTimeOff(id); }
  return { week, holidays, loadWeek, setWeek, loadHolidays, addHoliday, removeHoliday, addTimeOff, removeTimeOff };
});
