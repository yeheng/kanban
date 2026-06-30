<script setup lang="ts">
import { computed, ref } from "vue";
import { useGanttStore } from "../stores/gantt";
import { fmtDate, parseDateStrict } from "../utils/date";
import type { GanttBar } from "../types";

const DAY_W = 28; // px per day
const props = defineProps<{ start: string; end: string }>();
const gantt = useGanttStore();

// Parse/format consistently in local time (see utils/date): mixing UTC `Date.parse` with a
// local formatter shifted dates by a day in zones west of UTC, corrupting drag/resize writes.
const startMs = computed(() => parseDateStrict(props.start));
const totalDays = computed(() => Math.max(1, Math.round((parseDateStrict(props.end) - startMs.value) / 86400000) + 1));
const days = computed(() => {
  const out: string[] = [];
  const d = new Date(startMs.value);
  for (let i = 0; i < totalDays.value; i++) { out.push(fmtDate(d.getTime())); d.setDate(d.getDate() + 1); }
  return out;
});

function dayIndexOf(dateStr: string) { return Math.round((parseDateStrict(dateStr) - startMs.value) / 86400000); }
function barLeft(b: GanttBar) { return dayIndexOf(b.start_date) * DAY_W; }
function barWidth(b: GanttBar) { return (dayIndexOf(b.end_date) - dayIndexOf(b.start_date) + 1) * DAY_W; }

const rows = computed(() => {
  const m = new Map<number, { resource_id: number; resource_name: string; bars: GanttBar[] }>();
  for (const b of gantt.bars) {
    if (!m.has(b.resource_id)) m.set(b.resource_id, { resource_id: b.resource_id, resource_name: b.resource_name, bars: [] });
    m.get(b.resource_id)!.bars.push(b);
  }
  return [...m.values()];
});

type Drag = { id: number; mode: "move" | "resize"; startX: number; origStart: string; origEnd: string; percent: number };
const drag = ref<Drag | null>(null);
const previewDelta = ref(0);

function toStr(ms: number) { return fmtDate(ms); }
function onDown(e: PointerEvent, b: GanttBar, mode: "move" | "resize") {
  const target = e.target as HTMLElement;
  if (mode === "resize" && !target.classList.contains("gantt-timeline__resize")) return;
  (e.target as HTMLElement).setPointerCapture(e.pointerId);
  drag.value = { id: b.allocation_id, mode, startX: e.clientX, origStart: b.start_date, origEnd: b.end_date, percent: b.percent };
  previewDelta.value = 0;
}
function onMove(e: PointerEvent) {
  if (!drag.value) return;
  previewDelta.value = Math.round((e.clientX - drag.value.startX) / DAY_W);
}
function onUp() {
  const d = drag.value; if (!d) return;
  const deltaMs = previewDelta.value * 86400000;
  const newStart = d.mode === "move" ? toStr(parseDateStrict(d.origStart) + deltaMs) : d.origStart;
  const newEnd = toStr(parseDateStrict(d.origEnd) + deltaMs);
  drag.value = null; previewDelta.value = 0;
  if ((newStart !== d.origStart || newEnd !== d.origEnd) && newStart <= newEnd) {
    gantt.moveOrResize(d.id, newStart, newEnd, d.percent);
  }
}

type Arrow = { x1: number; y1: number; x2: number; y2: number };
const arrows = computed<Arrow[]>(() => {
  const pos = new Map<number, { startX: number; endX: number; y: number; startMs: number }>();
  let rowIdx = 0;
  for (const r of rows.value) {
    for (const b of r.bars) {
      const left = barLeft(b);
      const startMs = parseDateStrict(b.start_date);
      const prev = pos.get(b.task_id);
      if (!prev || startMs < prev.startMs) {
        pos.set(b.task_id, { startX: left, endX: left + barWidth(b), y: rowIdx * 32 + 16, startMs });
      }
    }
    rowIdx++;
  }
  const out: Arrow[] = [];
  for (const e of gantt.deps) {
    const p = pos.get(e.predecessor_id); const s = pos.get(e.task_id);
    if (p && s) out.push({ x1: p.endX, y1: p.y, x2: s.startX, y2: s.y });
  }
  return out;
});
</script>

<template>
  <div class="gantt-timeline" @pointermove="onMove" @pointerup="onUp">
    <div class="gantt-timeline__axis">
      <div
        v-for="d in days"
        :key="d"
        class="gantt-timeline__day"
        :style="{ width: DAY_W + 'px' }"
      >{{ d.slice(8) }}</div>
    </div>
    <div class="gantt-timeline__rows" :style="{ width: totalDays * DAY_W + 'px' }">
      <div v-for="r in rows" :key="r.resource_id" class="gantt-timeline__row">
        <div class="gantt-timeline__res">{{ r.resource_name }}</div>
        <div class="gantt-timeline__track">
          <div
            v-for="b in r.bars"
            :key="b.allocation_id"
            class="gantt-timeline__bar"
            :style="{ left: barLeft(b) + 'px', width: barWidth(b) + 'px', opacity: drag?.id === b.allocation_id ? 0.5 : 1 }"
            @pointerdown.stop="(e) => onDown(e, b, 'move')"
          >
            {{ b.task_title }} · {{ Math.round(b.percent * 100) }}%
            <span class="gantt-timeline__resize" @pointerdown.stop="(e) => onDown(e, b, 'resize')">⇔</span>
          </div>
        </div>
      </div>
      <svg class="gantt-timeline__arrows" :width="totalDays * DAY_W" :height="rows.length * 32">
        <line v-for="(a, i) in arrows" :key="i" :x1="a.x1" :y1="a.y1" :x2="a.x2" :y2="a.y2" stroke="#888" stroke-width="1" marker-end="url(#gantt-arrow)" />
        <defs>
          <marker id="gantt-arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
            <path d="M0,0 L6,3 L0,6 Z" fill="#888" />
          </marker>
        </defs>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.gantt-timeline {
  overflow-x: auto;
}
.gantt-timeline__axis {
  display: flex;
  position: sticky;
  top: 0;
  background: #fff;
  border-bottom: 1px solid #eee;
  z-index: 1;
}
.gantt-timeline__day {
  font-size: 10px;
  color: #888;
  border-right: 1px solid #f0f0f0;
  text-align: center;
}
.gantt-timeline__rows {
  position: relative;
}
.gantt-timeline__row {
  height: 32px;
  border-bottom: 1px solid #f5f5f5;
  display: flex;
  align-items: center;
}
.gantt-timeline__res {
  width: 100px;
  min-width: 100px;
  font-size: 12px;
  padding-left: 4px;
  background: #fff;
  z-index: 1;
}
.gantt-timeline__track {
  position: relative;
  height: 32px;
  flex: 1;
}
.gantt-timeline__bar {
  position: absolute;
  top: 4px;
  height: 24px;
  background: #2080f0;
  color: #fff;
  border-radius: 4px;
  font-size: 11px;
  line-height: 24px;
  padding: 0 6px;
  cursor: grab;
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
}
.gantt-timeline__resize {
  position: absolute;
  right: 0;
  top: 0;
  width: 12px;
  cursor: ew-resize;
  text-align: center;
}
.gantt-timeline__arrows {
  position: absolute;
  top: 28px;
  left: 100px;
  pointer-events: none;
}
</style>
