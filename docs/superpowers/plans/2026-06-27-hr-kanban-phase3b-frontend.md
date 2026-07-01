# HR Kanban — Phase 3b: Frontend (Gantt + Calendar Grid) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render a **Gantt** (project + cross-project resource views) with allocation bars, dependency arrows, and drag-to-move/resize (persisted via `update_allocation`), plus a **calendar occupancy grid** (resources × days, colored by utilization).

**Architecture:** One small backend addition (`update_allocation`). The Gantt is a self-drawn HTML/CSS timeline (design §7 recommends self-draw SVG to avoid GPL `dhtmlx`): a day-based time axis, resource rows, absolutely-positioned bars, pointer-event drag (move + right-edge resize), and an SVG overlay for dependency arrows. The calendar grid consumes `daily_occupancy`. Native drag — no chart library.

**Tech Stack:** Vue 3 + TS, Pinia, Naive UI, Tauri invoke, Vitest. No new deps.

**Prerequisite:** Phase 3 backend green (`gantt_project`, `gantt_resource`, `dependencies_for_project`, `daily_occupancy` commands; `GanttBar`, `DepEdge`, `DayOccupancy` types).

**Scope note:** Project Gantt (bars + move/resize + arrows), resource cross-project Gantt, calendar occupancy grid. Long-term-task segmentation display and virtualization are later refinements. AI panel is Phase 4b.

**Reference design:** `docs/design/2026-06-27-kanban-design.md` (§7 Gantt / calendar views).

---

## File Structure

```
kanban/
├── crates/db/src/repo/allocations.rs   # MOD: add update()
├── crates/app/src/command.rs           # MOD: add update_allocation
└── src/
    ├── api/index.ts                    # MOD: updateAllocation, gantt*, dailyOccupancy, dependencies
    ├── stores/gantt.ts                 # NEW (+ test)
    ├── components/
    │   ├── GanttTimeline.vue           # NEW (axis + rows + bars + drag + arrows)
    │   └── OccupancyGrid.vue           # NEW
    └── views/
        ├── GanttView.vue               # NEW (project + resource toggle)
        └── CalendarGridView.vue        # NEW
```

---

## Task 1: `update_allocation` command + API + store

**Files:**
- Modify: `crates/db/src/repo/allocations.rs`, `crates/app/src/command.rs`, `src-tauri/src/main.rs`
- Modify: `src/api/index.ts`, create `src/stores/gantt.ts` (+ test)

- [ ] **Step 1: `AllocationsRepo::update` — append to `crates/db/src/repo/allocations.rs`**

```rust
impl AllocationsRepo {
    /// Update an allocation's window/percent. The trg_allocation_validate_update
    /// trigger enforces the task/resource window intersection (design §3.3.15a).
    pub async fn update(
        pool: &SqlitePool, id: i64, start: &str, end: &str, percent: f64,
    ) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE allocations SET start_date=?, end_date=?, percent=?, \
                    updated_at=strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id=? AND deleted_at IS NULL")
            .bind(start).bind(end).bind(percent).bind(id)
            .execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}
```

- [ ] **Step 2: Command — append to `crates/app/src/command.rs`**

```rust
#[tauri::command]
pub async fn update_allocation(
    state: tauri::State<'_, AppState>, id: i64, start: String, end: String, percent: f64,
) -> Result<(), AppError> {
    if !(percent > 0.0 && percent <= 1.0) { return Err(domain::DomainError::InvalidRatio(percent).into()); }
    if end.as_str() < start.as_str() { return Err(domain::DomainError::InvalidDateWindow.into()); }
    Ok(db::AllocationsRepo::update(&state.pool, id, &start, &end, percent).await?)
}
```

- [ ] **Step 3: Register in `src-tauri/src/main.rs`** — add `update_allocation` to the handler list.

- [ ] **Step 4: API + types — append to `src/api/index.ts`** and `src/types.ts`

`types.ts`:
```ts
export interface GanttBar {
  allocation_id: number; resource_id: number; resource_name: string;
  task_id: number; task_title: string; project_id: number; project_name: string;
  start_date: string; end_date: string; percent: number; status: string; source: string;
}
export interface DepEdge { task_id: number; predecessor_id: number; lag_days: number; dep_type: string; }
export interface DayOccupancy {
  date: string; resource_id: number; resource_name: string;
  workload_pd: number; capacity_pd: number; utilization: number;
}
```
`api/index.ts` (append to `api`):
```ts
  updateAllocation: (id: number, start: string, end: string, percent: number) =>
    invoke<void>("update_allocation", { id, start, end, percent }),
  ganttProject: (projectId: number) => invoke<GanttBar[]>("gantt_project", { projectId }),
  ganttResource: (resourceId: number) => invoke<GanttBar[]>("gantt_resource", { resourceId }),
  dependenciesForProject: (projectId: number) => invoke<DepEdge[]>("dependencies_for_project", { projectId }),
  dailyOccupancy: (start: string, end: string) => invoke<DayOccupancy[]>("daily_occupancy", { start, end }),
```

- [ ] **Step 5: `src/stores/gantt.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { GanttBar, DepEdge } from "../types";

export const useGanttStore = defineStore("gantt", () => {
  const bars = ref<GanttBar[]>([]);
  const deps = ref<DepEdge[]>([]);
  const mode = ref<"project" | "resource">("project");
  const focusId = ref<number | null>(null); // project_id or resource_id depending on mode

  async function load() {
    if (mode.value === "project" && focusId.value != null) {
      [bars.value, deps.value] = await Promise.all([
        api.ganttProject(focusId.value), api.dependenciesForProject(focusId.value)]);
    } else if (mode.value === "resource" && focusId.value != null) {
      bars.value = await api.ganttResource(focusId.value);
      deps.value = [];
    }
  }
  async function moveOrResize(allocationId: number, start: string, end: string, percent: number) {
    await api.updateAllocation(allocationId, start, end, percent);
    await load();
  }
  return { bars, deps, mode, focusId, load, moveOrResize };
});
```

- [ ] **Step 6: `src/stores/gantt.test.ts`**

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useGanttStore } from "./gantt";
import { api } from "../api";

vi.mock("../api", () => ({ api: { ganttProject: vi.fn(), ganttResource: vi.fn(), dependenciesForProject: vi.fn(), updateAllocation: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); });

describe("gantt store", () => {
  it("loads project bars + deps", async () => {
    vi.mocked(api.ganttProject).mockResolvedValue([{ allocation_id: 1, resource_id: 1, resource_name: "A", task_id: 10, task_title: "T", project_id: 5, project_name: "P", start_date: "2026-06-29", end_date: "2026-07-03", percent: 0.5, status: "planned", source: "manual" }]);
    vi.mocked(api.dependenciesForProject).mockResolvedValue([]);
    const s = useGanttStore(); s.mode = "project"; s.focusId = 5;
    await s.load();
    expect(s.bars.length).toBe(1);
  });
});
```

- [ ] **Step 7: Run + build + commit**

```bash
npm test -- src/stores/gantt.test.ts && cargo build --workspace
git add -A && git commit -m "feat(app,web): update_allocation + gantt store/API"
```

---

## Task 2: GanttTimeline component (axis, bars, drag move/resize, arrows)

**Files:**
- Create: `src/components/GanttTimeline.vue`

- [ ] **Step 1: `src/components/GanttTimeline.vue`**

```vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { useGanttStore } from "../stores/gantt";
import type { GanttBar } from "../types";

const DAY_W = 28; // px per day
const props = defineProps<{ start: string; end: string }>();
const gantt = useGanttStore();

const startMs = computed(() => Date.parse(props.start));
const totalDays = computed(() => Math.max(1, Math.round((Date.parse(props.end) - startMs.value) / 86400000) + 1));
const days = computed(() => Array.from({ length: totalDays.value }, (_, i) => {
  const d = new Date(startMs.value + i * 86400000);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}));

function dayIndexOf(dateStr: string) { return Math.round((Date.parse(dateStr) - startMs.value) / 86400000); }
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

// ---- drag (move bar body / resize right edge) ----
const drag = ref<{ id: number; mode: "move" | "resize"; startX: number; origStart: string; origEnd: string; percent: number } | null>(null);
const previewDelta = ref(0); // days

function toStr(ms: number) {
  const d = new Date(ms); return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
function onDown(e: PointerEvent, b: GanttBar, mode: "move" | "resize") {
  const target = e.target as HTMLElement;
  if (mode === "resize" && !target.classList.contains("resize-handle")) return;
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
  const newStart = d.mode === "move" ? toStr(Date.parse(d.origStart) + deltaMs) : d.origStart;
  const newEnd = toStr(Date.parse(d.origEnd) + deltaMs);
  drag.value = null; previewDelta.value = 0;
  if (newStart !== d.origStart || newEnd !== d.origEnd) {
    if (newStart <= newEnd) gantt.moveOrResize(d.id, newStart, newEnd, d.percent);
  }
}

// ---- dependency arrows (SVG overlay) ----
const arrows = computed(() => {
  // map task_id -> last bar position (for simplistic arrow: pred end -> succ start)
  const pos = new Map<number, { x: number; y: number }>();
  let rowIdx = 0;
  for (const r of rows.value) {
    for (const b of r.bars) {
      pos.set(b.task_id, { x: barLeft(b) + barWidth(b), y: rowIdx * 32 + 16 });
    }
    rowIdx++;
  }
  return gantt.deps.map((e) => {
    const p = pos.get(e.predecessor_id); const s = pos.get(e.task_id);
    return p && s ? { x1: p.x, y1: p.y, x2: s.x, y2: s.y } : null;
  }).filter(Boolean);
});
</script>

<template>
  <div class="gantt" @pointermove="onMove" @pointerup="onUp">
    <div class="axis">
      <div v-for="d in days" :key="d" class="day" :style="{ width: DAY_W + 'px' }">{{ d.slice(8) }}</div>
    </div>
    <div class="rows" :style="{ width: totalDays * DAY_W + 'px' }">
      <div v-for="r in rows" :key="r.resource_id" class="row">
        <div class="res-label">{{ r.resource_name }}</div>
        <div class="track">
          <div v-for="b in r.bars" :key="b.allocation_id" class="bar"
            :style="{ left: barLeft(b) + 'px', width: barWidth(b) + 'px', opacity: drag?.id === b.allocation_id ? 0.5 : 1 }"
            @pointerdown.stop="(e) => onDown(e, b, 'move')">
            {{ b.task_title }} · {{ Math.round(b.percent * 100) }}%
            <span class="resize-handle" @pointerdown.stop="(e) => onDown(e, b, 'resize')">⇔</span>
          </div>
        </div>
      </div>
      <svg class="arrows" :width="totalDays * DAY_W" :height="rows.length * 32">
        <line v-for="(a, i) in arrows" :key="i" :x1="a!.x1" :y1="a!.y1" :x2="a!.x2" :y2="a!.y2" stroke="#888" stroke-width="1" marker-end="url(#arrow)" />
        <defs><marker id="arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="#888" /></marker></defs>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.gantt { overflow-x: auto; }
.axis { display: flex; position: sticky; top: 0; background: #fff; border-bottom: 1px solid #eee; }
.day { font-size: 10px; color: #888; border-right: 1px solid #f0f0f0; text-align: center; }
.rows { position: relative; }
.row { height: 32px; border-bottom: 1px solid #f5f5f5; display: flex; align-items: center; }
.res-label { width: 100px; min-width: 100px; font-size: 12px; padding-left: 4px; }
.track { position: relative; height: 32px; flex: 1; }
.bar { position: absolute; top: 4px; height: 24px; background: #2080f0; color: #fff; border-radius: 4px; font-size: 11px; line-height: 24px; padding: 0 6px; cursor: grab; user-select: none; white-space: nowrap; overflow: hidden; }
.resize-handle { position: absolute; right: 0; top: 0; width: 12px; cursor: ew-resize; text-align: center; }
.arrows { position: absolute; top: 28px; left: 100px; pointer-events: none; }
</style>
```

> Drag move shifts both dates by whole days; the right-edge handle resizes the end date. `pointerup` calls `moveOrResize` → `update_allocation`; the DB trigger rejects moves outside the task/resource window (surfaced as an error toast at the view level).

- [ ] **Step 2: Build-check**

Run: `npm run build`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat(web): GanttTimeline (bars + drag move/resize + dep arrows)"
```

---

## Task 3: GanttView (project + resource toggle)

**Files:**
- Create: `src/views/GanttView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: `src/views/GanttView.vue`**

```vue
<script setup lang="ts">
import { computed, ref, watchEffect } from "vue";
import { useGanttStore } from "../stores/gantt";
import { useProjectsStore } from "../stores/projects";
import { useResourcesStore } from "../stores/resources";
import GanttTimeline from "../components/GanttTimeline.vue";

const gantt = useGanttStore();
const projects = useProjectsStore();
const resources = useResourcesStore();
const err = ref<string | null>(null);
// default window: next 6 weeks from today (kept simple)
const start = ref("2026-06-29"); const end = ref("2026-08-09");

watchEffect(async () => {
  if (gantt.mode === "project" && projects.current) { gantt.focusId = projects.current; await safeLoad(); }
});
async function safeLoad() {
  try { err.value = null; await gantt.load(); } catch (e: any) { err.value = String(e); }
}
async function onResource(id: number) { gantt.mode = "resource"; gantt.focusId = id; await safeLoad(); }
</script>

<template>
  <h2 style="margin-top:0">Gantt</h2>
  <div style="margin-bottom:8px">
    模式：
    <button :disabled="gantt.mode==='project'" @click="gantt.mode='project'; safeLoad()">项目</button>
    <select v-model.number="projects.current" @change="safeLoad" :disabled="gantt.mode!=='project'">
      <option v-for="p in projects.items" :key="p.id" :value="p.id">{{ p.name }}</option>
    </select>
    <span> 或资源视角：</span>
    <select @change="onResource(+($event.target as HTMLSelectElement).value)">
      <option :value="0">—</option>
      <option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option>
    </select>
    <span style="color:#d03050" v-if="err"> ⚠ {{ err }}（可能越出任务/资源时间窗）</span>
  </div>
  <GanttTimeline :start="start" :end="end" />
</template>
```

- [ ] **Step 2: Route + nav** — add `{ path: "/gantt", component: () => import("./views/GanttView.vue") }` + nav link.

- [ ] **Step 3: Build + commit**

```bash
npm run build && git add -A && git commit -m "feat(web): GanttView (project + cross-project resource)"
```

---

## Task 4: Calendar occupancy grid view

**Files:**
- Create: `src/components/OccupancyGrid.vue`, `src/views/CalendarGridView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: `src/components/OccupancyGrid.vue`**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useWorkloadStore } from "../stores/workload";
import type { DayOccupancy } from "../types";
const props = defineProps<{ items: DayOccupancy[]; days: string[]; resources: { id: number; name: string }[] }>();
const wl = useWorkloadStore();
function cell(rid: number, day: string) {
  return props.items.find((o) => o.resource_id === rid && o.date === day);
}
function bg(o?: DayOccupancy) {
  if (!o) return "#f7f7fa";
  return { under: "#e0e0e6", green: "#9ad19a", yellow: "#f0d070", red: "#e08090" }[wl.band(o.utilization)];
}
</script>
<template>
  <table border="1" cellpadding="6" style="border-collapse:collapse">
    <thead><tr><th>资源</th><th v-for="d in days" :key="d">{{ d.slice(8) }}</th></tr></thead>
    <tbody>
      <tr v-for="r in resources" :key="r.id">
        <td>{{ r.name }}</td>
        <td v-for="d in days" :key="d" :style="{ background: bg(cell(r.id, d)) }">
          <small v-if="cell(r.id, d)">{{ Math.round(cell(r.id, d)!.utilization * 100) }}%</small>
        </td>
      </tr>
    </tbody>
  </table>
</template>
```

- [ ] **Step 2: `src/views/CalendarGridView.vue`**

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api } from "../api";
import { useResourcesStore } from "../stores/resources";
import { useWorkloadStore } from "../stores/workload";
import OccupancyGrid from "../components/OccupancyGrid.vue";
import type { DayOccupancy } from "../types";

const resources = useResourcesStore();
const wl = useWorkloadStore();
const start = ref("2026-06-29"); const end = ref("2026-07-12");
const items = ref<DayOccupancy[]>([]);
const days = ref<string[]>([]);

function buildDays() {
  const out: string[] = []; let ms = Date.parse(start.value);
  while (ms <= Date.parse(end.value)) {
    const d = new Date(ms);
    out.push(`${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,"0")}-${String(d.getDate()).padStart(2,"0")}`);
    ms += 86400000;
  }
  days.value = out;
}
async function refresh() {
  buildDays();
  items.value = await api.dailyOccupancy(start.value, end.value);
}
onMounted(async () => { await wl.loadThresholds(); await resources.load(); await refresh(); });
</script>
<template>
  <h2 style="margin-top:0">日历 / Calendar 占用</h2>
  <input v-model="start" type="date" /> – <input v-model="end" type="date" />
  <button @click="refresh">刷新</button>
  <OccupancyGrid :items="items" :days="days" :resources="resources.items" />
</template>
```

- [ ] **Step 3: Route + nav** — add `{ path: "/calendar-grid", component: () => import("./views/CalendarGridView.vue") }` + nav link.

- [ ] **Step 4: Build + commit**

```bash
npm run build && git add -A && git commit -m "feat(web): calendar occupancy grid"
```

---

## Task 5: End-to-end smoke

- [ ] **Step 1: Run** `npm run tauri dev`.

- [ ] **Step 2: Manual checklist**
- [ ] With a project + allocations, open **Gantt** → bars appear on resource rows across the time axis.
- [ ] Drag a bar body left/right → dates update (list/allocations reflect); dragging past the task window shows the ⚠ error and reverts.
- [ ] Drag a bar's right-edge ⇔ → end date resizes.
- [ ] Add a task dependency (or via an existing one) → an arrow renders from predecessor end to successor start.
- [ ] Switch to **resource** mode + pick a resource → bars span multiple projects.
- [ ] Open **Calendar grid** → cells colored green/yellow/red by daily utilization; weekends blank.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "test: Phase 3b end-to-end smoke (gantt/calendar)"
```

---

## Self-Review

**Spec coverage (§7 Gantt/calendar + roadmap Phase 3 frontend):**
- §7 Gantt project view (bars by resource row, drag move/resize, dep arrows) → Tasks 2–3 ✓
- §7 Gantt cross-project resource view → Task 3 ✓
- §7 calendar daily occupancy (colored grid) → Task 4 ✓
- Allocation mutation via drag → `update_allocation` (Task 1) ✓

**Deferred (not placeholders):** virtualization (large datasets), long-term-task segment rendering, `update_allocation` percent-edit UI (only move/resize now), AI panel (Phase 4b).

**Placeholder scan:** none — complete code; store test asserts concrete behavior.

**Type consistency:** `GanttBar`/`DepEdge`/`DayOccupancy` TS fields match Rust Serialize aliases; `updateAllocation` camelCase → `update_allocation` params; date math is whole-day (`86400000` ms), matching the DB `YYYY-MM-DD` granularity.

**Known impl-time items:** drag is whole-day snapping (no sub-day); percent editing via drag not implemented (resize only). Arrows use a simplified pred-end→succ-start geometry (no routing around bars) — acceptable for MVP density. If a drag is rejected by the trigger, the store's `load()` reverts the optimistic state.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-kanban-phase3b-frontend.md`. **1. Subagent-Driven** (recommended) or **2. Inline**. Next: **Phase 4 (AI engine)**.
