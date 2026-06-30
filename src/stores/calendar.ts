import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Holiday, TimeOff, WeekTemplate } from "../types";
import { useRefreshStore } from "./refresh";

// A working-calendar change (week template, holiday, time-off) shifts capacity, so it
// invalidates utilization (workload) and the occupancy grid (calendar) (design G4).
function bumpCalendar() {
  useRefreshStore().bump("workload", "calendar");
}

function weekFromTemplate(t: WeekTemplate | undefined): number[] {
  if (!t) return [1, 1, 1, 1, 1, 0, 0];
  const f = (bit: number, frac: number) => (bit ? frac : 0);
  return [f(t.mon, t.mon_frac), f(t.tue, t.tue_frac), f(t.wed, t.wed_frac),
          f(t.thu, t.thu_frac), f(t.fri, t.fri_frac), f(t.sat, t.sat_frac), f(t.sun, t.sun_frac)];
}

export const useCalendarStore = defineStore("calendar", () => {
  const week = ref<number[]>([1, 1, 1, 1, 1, 0, 0]);
  const holidays = ref<Holiday[]>([]);
  const timeOff = ref<TimeOff[]>([]);

  async function loadWeek() {
    const rows = await api.listWorkWeeks();
    week.value = weekFromTemplate(rows.find((r) => r.scope === "global"));
  }
  async function setWeek(w: number[]) { week.value = w; await api.setGlobalWorkWeek(w); bumpCalendar(); }
  async function loadHolidays() { holidays.value = await api.listHolidays(); }
  async function addHoliday(day: string, fraction: number, name: string | null) { await api.addHoliday(null, day, fraction, name); await loadHolidays(); bumpCalendar(); }
  async function removeHoliday(id: number) { await api.deleteHoliday(id); await loadHolidays(); bumpCalendar(); }
  async function loadTimeOff() { timeOff.value = await api.listTimeOff(); }
  async function addTimeOff(resourceId: number, day: string, fraction: number, reason: string | null) {
    await api.addTimeOff(resourceId, day, fraction, reason);
    await loadTimeOff();
    bumpCalendar();
  }
  async function removeTimeOff(id: number) { await api.deleteTimeOff(id); await loadTimeOff(); bumpCalendar(); }
  return { week, holidays, timeOff, loadWeek, setWeek, loadHolidays, addHoliday, removeHoliday, loadTimeOff, addTimeOff, removeTimeOff };
});
