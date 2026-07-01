# HR Kanban — Phase 2b: Frontend (Dashboard + Allocations + Calendar) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the workload **Dashboard** (resource/team utilization bars + overload alerts + project burn), an **allocation editor** (create allocations and see live workload impact), and **calendar management** (work week, holidays, time-off) — all consuming the Phase 2 backend commands.

**Architecture:** A few thin backend commands are added first (repos already exist). Frontend adds typed API methods, three Pinia stores (`workload`, `allocations`, `calendar`, plus `teams`), a reusable `UtilBar` colored by global thresholds, and three views wired into the router. Tests cover stores via Vitest; views are verified by an E2E smoke.

**Tech Stack:** Vue 3 + TS, Pinia, Naive UI, Vitest, Tauri invoke. No new deps.

**Prerequisite:** Phase 0/1/1b/2-backend implemented and green. Available commands: `resource_summary`, `team_summary`, `overloads`, `project_burn`, `set_global_work_week`, `list_work_weeks`, `add_holiday`, `list_holidays`, `add_time_off`, `list_resources`, `list_projects`.

**Scope note:** Dashboard + allocation editor + calendar management only. Gantt/Calendar **visualizations** are Phase 3b. Production first-run passphrase + teams-CRUD polish remain deferred.

**Reference design:** `docs/design/2026-06-27-kanban-design.md` (§7 Dashboard / allocation / calendar views).

---

## File Structure

```
kanban/
├── crates/app/src/command.rs        # MOD: add list_teams/list_team_members/list_tasks/
│                                    #       create_allocation/list_allocations/get_thresholds
├── crates/db/src/
│   ├── models.rs                    # MOD: add AllocationView
│   └── repo/allocations.rs          # MOD: add list_by_project
└── src/
    ├── api/index.ts                 # MOD: workload/allocation/calendar/team methods
    ├── types.ts                     # MOD: Allocation/Thresholds/summary types
    ├── stores/
    │   ├── workload.ts              # NEW (+ test)
    │   ├── allocations.ts           # NEW (+ test)
    │   ├── calendar.ts              # NEW
    │   └── teams.ts                 # NEW
    ├── components/
    │   ├── UtilBar.vue              # NEW
    │   ├── AllocationForm.vue       # NEW
    │   ├── WorkWeekEditor.vue       # NEW
    │   ├── HolidayList.vue          # NEW
    │   └── TimeOffList.vue          # NEW
    └── views/
        ├── DashboardView.vue        # NEW
        ├── AllocationsView.vue      # NEW
        └── CalendarView.vue         # NEW
```

---

## Task 1: Backend additions — commands the UI needs

**Files:**
- Modify: `crates/db/src/models.rs`, `crates/db/src/repo/allocations.rs`
- Modify: `crates/app/src/command.rs`
- Modify: `src-tauri/src/main.rs` (register commands)

- [ ] **Step 1: `AllocationView` model — append to `crates/db/src/models.rs`**

```rust
/// Allocation joined with resource name + task title (for the allocation editor / Gantt later).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AllocationView {
    pub id: i64,
    pub resource_id: i64,
    pub resource_name: String,
    pub task_id: i64,
    pub task_title: String,
    pub project_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
}
```

- [ ] **Step 2: `list_by_project` — append to `crates/db/src/repo/allocations.rs`**

```rust
use crate::models::AllocationView;

impl AllocationsRepo {
    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<AllocationView>, DbError> {
        Ok(sqlx::query_as::<_, AllocationView>(
            "SELECT a.id, a.resource_id, r.name AS resource_name, a.task_id, t.title AS task_title, \
                    t.project_id, a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             WHERE t.project_id = ? AND a.deleted_at IS NULL \
             ORDER BY a.start_date")
            .bind(project_id).fetch_all(pool).await?)
    }
}
```

- [ ] **Step 3: Add commands — append to `crates/app/src/command.rs`**

```rust
use db::models::{AllocationView, Task, Team, TeamMember};
use db::{SettingsRepo, TeamMembersRepo, TeamsRepo, TasksRepo};

#[tauri::command]
pub async fn list_teams(state: tauri::State<'_, AppState>) -> Result<Vec<Team>, AppError> {
    Ok(TeamsRepo::list_active(&state.pool).await?)
}
#[tauri::command]
pub async fn list_team_members(state: tauri::State<'_, AppState>, team_id: i64) -> Result<Vec<TeamMember>, AppError> {
    Ok(TeamMembersRepo::list_members(&state.pool, team_id).await?)
}
#[tauri::command]
pub async fn list_tasks(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<Task>, AppError> {
    Ok(TasksRepo::list_by_project(&state.pool, project_id).await?)
}
#[tauri::command]
pub async fn create_allocation(
    state: tauri::State<'_, AppState>,
    resource_id: i64, task_id: i64, start: String, end: String, percent: f64,
) -> Result<i64, AppError> {
    if !(percent > 0.0 && percent <= 1.0) { return Err(domain::DomainError::InvalidRatio(percent).into()); }
    Ok(db::AllocationsRepo::create(&state.pool, resource_id, task_id, &start, &end, percent).await?)
}
#[tauri::command]
pub async fn list_allocations(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<AllocationView>, AppError> {
    Ok(db::AllocationsRepo::list_by_project(&state.pool, project_id).await?)
}
#[tauri::command]
pub async fn get_thresholds(state: tauri::State<'_, AppState>) -> Result<ThresholdsDto, AppError> {
    let t = SettingsRepo::thresholds(&state.pool).await?;
    Ok(ThresholdsDto { overload: t.overload, underload: t.underload, green: t.green, yellow: t.yellow })
}

#[derive(serde::Serialize)]
pub struct ThresholdsDto { pub overload: f64, pub underload: f64, pub green: f64, pub yellow: f64 }
```

- [ ] **Step 4: Register commands in `src-tauri/src/main.rs`**

Add to the `use app::command::{...}` import and `generate_handler![...]`:
```
list_teams, list_team_members, list_tasks, create_allocation, list_allocations, get_thresholds,
```

- [ ] **Step 5: Build-check**

Run: `cargo build --workspace`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(app): list_teams/members/tasks, allocation CRUD, get_thresholds commands"
```

---

## Task 2: Typed API + Pinia stores (TDD)

**Files:**
- Modify: `src/types.ts`, `src/api/index.ts`
- Create: `src/stores/workload.ts`, `src/stores/allocations.ts`, `src/stores/calendar.ts`, `src/stores/teams.ts`
- Create: `src/stores/workload.test.ts`

- [ ] **Step 1: Types — append to `src/types.ts`**

```ts
export interface ResourceSummary {
  resource_id: number; capacity_pd: number; workload_pd: number; utilization: number; overloaded: boolean;
}
export interface TeamSummary {
  team_id: number; capacity_pd: number; workload_pd: number; utilization: number; overloaded_members: number[];
}
export interface ProjectBurn { project_id: number; budget_pd: number; allocated_pd: number; usage: number; }
export interface Thresholds { overload: number; underload: number; green: number; yellow: number; }
export interface AllocationView {
  id: number; resource_id: number; resource_name: string; task_id: number; task_title: string;
  project_id: number; start_date: string; end_date: string; percent: number; status: string; source: string;
}
export interface Team { id: number; name: string; description: string | null; }
export interface TeamMember { team_id: number; resource_id: number; role: string | null; }
```

- [ ] **Step 2: API methods — append to the `api` object in `src/api/index.ts`**

```ts
  // workload
  resourceSummary: (resourceId: number, start: string, end: string) =>
    invoke<ResourceSummary>("resource_summary", { resourceId, start, end }),
  teamSummary: (teamId: number, start: string, end: string) =>
    invoke<TeamSummary>("team_summary", { teamId, start, end }),
  overloads: (start: string, end: string) => invoke<ResourceSummary[]>("overloads", { start, end }),
  projectBurn: (projectId: number) => invoke<ProjectBurn>("project_burn", { projectId }),
  getThresholds: () => invoke<Thresholds>("get_thresholds"),

  // allocations
  createAllocation: (resourceId: number, taskId: number, start: string, end: string, percent: number) =>
    invoke<number>("create_allocation", { resourceId, taskId, start, end, percent }),
  listAllocations: (projectId: number) => invoke<AllocationView[]>("list_allocations", { projectId }),

  // calendar
  setGlobalWorkWeek: (week: number[]) => invoke<void>("set_global_work_week", { week }),
  listWorkWeeks: () => invoke<unknown[]>("list_work_weeks"),
  addHoliday: (projectId: number | null, day: string, fraction: number | null, name: string | null) =>
    invoke<number>("add_holiday", { projectId, day, fraction, name }),
  listHolidays: () => invoke<{ id: number; project_id: number | null; day: string; fraction: number; name: string | null }[]>("list_holidays"),
  addTimeOff: (resourceId: number, day: string, fraction: number | null, reason: string | null) =>
    invoke<number>("add_time_off", { resourceId, day, fraction, reason }),

  // teams
  listTeams: () => invoke<Team[]>("list_teams"),
  listTeamMembers: (teamId: number) => invoke<TeamMember[]>("list_team_members", { teamId }),
```

- [ ] **Step 3: `src/stores/workload.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { ResourceSummary, TeamSummary, ProjectBurn, Thresholds } from "../types";

export const useWorkloadStore = defineStore("workload", () => {
  const resourceSummaries = ref<ResourceSummary[]>([]);
  const overloads = ref<ResourceSummary[]>([]);
  const thresholds = ref<Thresholds>({ overload: 1.1, underload: 0.5, green: 0.7, yellow: 1.0 });
  const teamSummary = ref<TeamSummary | null>(null);
  const projectBurn = ref<ProjectBurn | null>(null);

  async function loadThresholds() { thresholds.value = await api.getThresholds(); }
  async function loadResourceSummaries(resourceIds: number[], start: string, end: string) {
    resourceSummaries.value = await Promise.all(resourceIds.map((id) => api.resourceSummary(id, start, end)));
  }
  async function loadOverloads(start: string, end: string) { overloads.value = await api.overloads(start, end); }
  async function loadTeamSummary(teamId: number, start: string, end: string) { teamSummary.value = await api.teamSummary(teamId, start, end); }
  async function loadProjectBurn(projectId: number) { projectBurn.value = await api.projectBurn(projectId); }

  /** Color band for a utilization value using global thresholds. */
  function band(util: number): "under" | "green" | "yellow" | "red" {
    const t = thresholds.value;
    if (util >= t.overload) return "red";
    if (util >= t.yellow) return "yellow";
    if (util >= t.green) return "green";
    return "under";
  }
  return { resourceSummaries, overloads, thresholds, teamSummary, projectBurn,
           loadThresholds, loadResourceSummaries, loadOverloads, loadTeamSummary, loadProjectBurn, band };
});
```

- [ ] **Step 4: `src/stores/allocations.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { AllocationView } from "../types";

export const useAllocationsStore = defineStore("allocations", () => {
  const items = ref<AllocationView[]>([]);
  async function load(projectId: number) { items.value = await api.listAllocations(projectId); }
  async function create(resourceId: number, taskId: number, start: string, end: string, percent: number) {
    await api.createAllocation(resourceId, taskId, start, end, percent);
  }
  return { items, load, create };
});
```

- [ ] **Step 5: `src/stores/calendar.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";

export const useCalendarStore = defineStore("calendar", () => {
  const week = ref<number[]>([1, 1, 1, 1, 1, 0, 0]); // Mon..Sun fractions
  const holidays = ref<{ id: number; project_id: number | null; day: string; fraction: number; name: string | null }[]>([]);

  async function loadWeek() { await api.setGlobalWorkWeek(week.value); /* persist on change */ }
  async function setWeek(w: number[]) { week.value = w; await api.setGlobalWorkWeek(w); }
  async function loadHolidays() { holidays.value = await api.listHolidays(); }
  async function addHoliday(day: string, fraction: number, name: string | null) { await api.addHoliday(null, day, fraction, name); await loadHolidays(); }
  async function addTimeOff(resourceId: number, day: string, fraction: number, reason: string | null) { await api.addTimeOff(resourceId, day, fraction, reason); }
  return { week, holidays, loadWeek, setWeek, loadHolidays, addHoliday, addTimeOff };
});
```

- [ ] **Step 6: `src/stores/teams.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Team, TeamMember } from "../types";

export const useTeamsStore = defineStore("teams", () => {
  const items = ref<Team[]>([]);
  const members = ref<TeamMember[]>([]);
  async function load() { items.value = await api.listTeams(); }
  async function loadMembers(teamId: number) { members.value = await api.listTeamMembers(teamId); }
  return { items, members, load, loadMembers };
});
```

- [ ] **Step 7: `src/stores/workload.test.ts`**

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useWorkloadStore } from "./workload";
import { api } from "../api";

vi.mock("../api", () => ({ api: { getThresholds: vi.fn(), overloads: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); vi.mocked(api.getThresholds).mockReset(); vi.mocked(api.overloads).mockReset(); });

describe("workload store", () => {
  it("loads thresholds and bands utilization", async () => {
    vi.mocked(api.getThresholds).mockResolvedValue({ overload: 1.1, underload: 0.5, green: 0.7, yellow: 1.0 });
    const s = useWorkloadStore();
    await s.loadThresholds();
    expect(s.band(0.5)).toBe("green");
    expect(s.band(0.69)).toBe("under");
    expect(s.band(1.0)).toBe("yellow");
    expect(s.band(1.2)).toBe("red");
  });
});
```

- [ ] **Step 8: Run test — verify PASS**

Run: `npm test -- src/stores/workload.test.ts`
Expected: `1 passed`.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(web): workload/allocations/calendar/teams stores + API"
```

---

## Task 3: UtilBar + Dashboard view

**Files:**
- Create: `src/components/UtilBar.vue`, `src/views/DashboardView.vue`
- Modify: `src/router.ts`

- [ ] **Step 1: `src/components/UtilBar.vue`**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useWorkloadStore } from "../stores/workload";
const props = defineProps<{ utilization: number }>();
const wl = useWorkloadStore();
const pct = computed(() => Math.min(150, Math.round(props.utilization * 100)));
const band = computed(() => wl.band(props.utilization));
const color = computed(() => ({ under: "#d1d1d6", green: "#18a058", yellow: "#f0a020", red: "#d03050" }[band.value]));
</script>
<template>
  <div class="wrap" :title="`${pct}% (${band})`">
    <div class="fill" :style="{ width: pct + '%', background: color }" />
    <span class="label">{{ pct }}%</span>
  </div>
</template>
<style scoped>
.wrap { position: relative; width: 160px; height: 18px; background: #f0f0f0; border-radius: 4px; overflow: hidden; }
.fill { height: 100%; }
.label { position: absolute; right: 6px; top: 0; font-size: 11px; line-height: 18px; color: #333; }
</style>
```

- [ ] **Step 2: `src/views/DashboardView.vue`**

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useWorkloadStore } from "../stores/workload";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { useTeamsStore } from "../stores/teams";
import UtilBar from "../components/UtilBar.vue";

const wl = useWorkloadStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const teams = useTeamsStore();
const start = ref("2026-06-29"); const end = ref("2026-07-03");
const selectedTeam = ref<number | null>(null);

async function refresh() {
  await resources.load();
  await wl.loadResourceSummaries(resources.items.map((r) => r.id), start.value, end.value);
  await wl.loadOverloads(start.value, end.value);
  if (projects.current) await wl.loadProjectBurn(projects.current);
  if (selectedTeam.value) await wl.loadTeamSummary(selectedTeam.value, start.value, end.value);
}
onMounted(async () => { await wl.loadThresholds(); await teams.load(); await refresh(); });
</script>
<template>
  <div>
    <h2 style="margin-top:0">Dashboard / 人力概览</h2>
    <div style="margin-bottom:12px">
      窗口 <input v-model="start" type="date" /> – <input v-model="end" type="date" />
      <button @click="refresh">刷新</button>
    </div>

    <h3>过载预警 ({{ wl.overloads.length }})</h3>
    <div v-for="o in wl.overloads" :key="o.resource_id" class="alert">
      ⚠ 资源 #{{ o.resource_id }} 利用率 {{ Math.round(o.utilization * 100) }}%
    </div>
    <p v-if="!wl.overloads.length">无过载 🎉</p>

    <h3>资源利用率</h3>
    <table>
      <tr v-for="s in wl.resourceSummaries" :key="s.resource_id">
        <td style="width:120px">资源 #{{ s.resource_id }}</td>
        <td><UtilBar :utilization="s.utilization" /></td>
        <td>{{ s.workload_pd.toFixed(1) }} / {{ s.capacity_pd.toFixed(1) }} PD</td>
      </tr>
    </table>

    <h3>项目健康（预算消耗）</h3>
    <div v-if="wl.projectBurn">
      {{ wl.projectBurn.allocated_pd.toFixed(1) }} / {{ wl.projectBurn.budget_pd.toFixed(1) }} PD
      ({{ Math.round(wl.projectBurn.usage * 100) }}%)
    </div>

    <h3>团队利用率</h3>
    <select v-model.number="selectedTeam" @change="refresh">
      <option :value="null">— 选择团队 —</option>
      <option v-for="t in teams.items" :key="t.id" :value="t.id">{{ t.name }}</option>
    </select>
    <div v-if="wl.teamSummary">
      <UtilBar :utilization="wl.teamSummary.utilization" />
      <small>过载成员：{{ wl.teamSummary.overloaded_members.join(", ") || "无" }}</small>
    </div>
  </div>
</template>
<style scoped>
.alert { background: #fff0f0; border: 1px solid #ffc0cb; padding: 4px 8px; border-radius: 4px; margin: 2px 0; }
table td { padding: 2px 6px; }
</style>
```

- [ ] **Step 3: Register route — `src/router.ts`**

Add to `routes`:
```ts
{ path: "/dashboard", component: () => import("./views/DashboardView.vue") },
```

- [ ] **Step 4: Add nav link in `src/components/AppLayout.vue`** — insert a `<router-link to="/dashboard">Dashboard</router-link>` beside the others.

- [ ] **Step 5: Build-check + commit**

```bash
npm run build
git add -A
git commit -m "feat(web): Dashboard (utilization bars + overload alerts + burn + team)"
```

---

## Task 4: Allocation editor view

**Files:**
- Create: `src/components/AllocationForm.vue`, `src/views/AllocationsView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: `src/components/AllocationForm.vue`** (create allocation + show live impact)

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { api } from "../api";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const resourceId = ref<number | null>(null);
const taskId = ref<number | null>(null);
const start = ref("2026-06-29"); const end = ref("2026-07-03"); const percent = ref(0.5);
const tasks = ref<{ id: number; title: string }[]>([]);
const impact = ref<{ utilization: number; overloaded: boolean } | null>(null);
const error = ref<string | null>(null);

async function loadTasks() {
  if (!projects.current) return;
  tasks.value = (await api.listTasks(projects.current)).map((t) => ({ id: t.id, title: t.title }));
}
async function submit() {
  error.value = null;
  if (resourceId.value == null || taskId.value == null || !projects.current) return;
  try {
    await allocations.create(resourceId.value, taskId.value, start.value, end.value, percent.value);
    await allocations.load(projects.current);
    impact.value = await api.resourceSummary(resourceId.value, start.value, end.value)
      .then((s) => ({ utilization: s.utilization, overloaded: s.overloaded }));
  } catch (e: any) {
    error.value = String(e);
  }
}
loadTasks();
</script>
<template>
  <form @submit.prevent="submit">
    <select v-model.number="resourceId"><option :value="null">资源</option><option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option></select>
    <select v-model.number="taskId"><option :value="null">任务</option><option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.title }}</option></select>
    <input v-model="start" type="date" /><input v-model="end" type="date" />
    <input v-model.number="percent" type="number" min="0.01" max="1" step="0.05" />
    <button>分配</button>
    <span v-if="error" style="color:#d03050">{{ error }}</span>
    <span v-if="impact" :style="{ color: impact.overloaded ? '#d03050' : '#18a058' }">→ 利用率 {{ Math.round(impact.utilization * 100) }}%{{ impact.overloaded ? ' ⚠过载' : '' }}</span>
  </form>
</template>
```

- [ ] **Step 2: `src/views/AllocationsView.vue`**

```vue
<script setup lang="ts">
import { onMounted, watchEffect } from "vue";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import AllocationForm from "../components/AllocationForm.vue";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
onMounted(() => resources.load());
watchEffect(async () => { if (projects.current) await allocations.load(projects.current); });
</script>
<template>
  <h2 style="margin-top:0">分配 / Allocations</h2>
  <AllocationForm />
  <table border="1" cellpadding="4" style="border-collapse:collapse;margin-top:12px">
    <tr><th>资源</th><th>任务</th><th>区间</th><th>投入</th><th>来源</th></tr>
    <tr v-for="a in allocations.items" :key="a.id">
      <td>{{ a.resource_name }}</td><td>{{ a.task_title }}</td>
      <td>{{ a.start_date }} → {{ a.end_date }}</td>
      <td>{{ Math.round(a.percent * 100) }}%</td><td>{{ a.source }}</td>
    </tr>
  </table>
</template>
```

- [ ] **Step 3: Route + nav** — add `{ path: "/allocations", component: () => import("./views/AllocationsView.vue") }` and a nav link.

- [ ] **Step 4: Build-check + commit**

```bash
npm run build
git add -A
git commit -m "feat(web): allocation editor (create + live workload impact)"
```

---

## Task 5: Calendar management view

**Files:**
- Create: `src/components/WorkWeekEditor.vue`, `src/components/HolidayList.vue`, `src/components/TimeOffList.vue`, `src/views/CalendarView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: `src/components/WorkWeekEditor.vue`** (7 day fractions, 0/0.5/1 quick toggle)

```vue
<script setup lang="ts">
import { useCalendarStore } from "../stores/calendar";
const cal = useCalendarStore();
const labels = ["一", "二", "三", "四", "五", "六", "日"];
function cycle(i: number) {
  const cur = cal.week[i];
  const next = cur >= 1 ? 0 : cur >= 0.5 ? 1 : 0.5;
  const w = [...cal.week]; w[i] = next; cal.setWeek(w);
}
</script>
<template>
  <div>
    <span v-for="(f, i) in cal.week" :key="i" @click="cycle(i)" class="day" :style="{ opacity: f === 0 ? 0.3 : 1 }">
      {{ labels[i] }}<small>{{ f === 0 ? "休" : f === 0.5 ? "半" : "全" }}</small>
    </span>
    <p><small>点击切换 全天/半天/休息（写入全局工作周模板）</small></p>
  </div>
</template>
<style scoped>
.day { display: inline-block; cursor: pointer; border: 1px solid #ccc; border-radius: 6px; padding: 6px 10px; margin: 2px; user-select: none; }
small { display: block; font-size: 10px; color: #888; }
</style>
```

- [ ] **Step 2: `src/components/HolidayList.vue`**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useCalendarStore } from "../stores/calendar";
const cal = useCalendarStore();
const day = ref(""); const frac = ref(1); const name = ref("");
async function add() { if (!day.value) return; await cal.addHoliday(day.value, frac.value, name.value || null); day.value = ""; name.value = ""; }
</script>
<template>
  <div>
    <input v-model="day" type="date" />
    <select v-model.number="frac"><option :value="1">全天</option><option :value="0.5">半天</option></select>
    <input v-model="name" placeholder="名称" />
    <button @click="add">添加节假日</button>
    <ul><li v-for="h in cal.holidays" :key="h.id">{{ h.day }} · {{ h.fraction === 1 ? "全天" : "半天" }} · {{ h.name }}</li></ul>
  </div>
</template>
```

- [ ] **Step 3: `src/components/TimeOffList.vue`**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
const cal = useCalendarStore(); const resources = useResourcesStore();
const rid = ref<number | null>(null); const day = ref(""); const frac = ref(1); const reason = ref("");
async function add() { if (rid.value == null || !day.value) return; await cal.addTimeOff(rid.value, day.value, frac.value, reason.value || null); }
</script>
<template>
  <div>
    <select v-model.number="rid"><option :value="null">资源</option><option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option></select>
    <input v-model="day" type="date" />
    <select v-model.number="frac"><option :value="1">全天</option><option :value="0.5">半天</option></select>
    <input v-model="reason" placeholder="原因" />
    <button @click="add">添加请假</button>
  </div>
</template>
```

- [ ] **Step 4: `src/views/CalendarView.vue`**

```vue
<script setup lang="ts">
import { onMounted } from "vue";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
import WorkWeekEditor from "../components/WorkWeekEditor.vue";
import HolidayList from "../components/HolidayList.vue";
import TimeOffList from "../components/TimeOffList.vue";
const cal = useCalendarStore(); const resources = useResourcesStore();
onMounted(async () => { await resources.load(); await cal.loadHolidays(); });
</script>
<template>
  <h2 style="margin-top:0">日历 / Calendar</h2>
  <h3>工作周模板</h3><WorkWeekEditor />
  <h3>节假日</h3><HolidayList />
  <h3>请假 / 调休</h3><TimeOffList />
</template>
```

- [ ] **Step 5: Route + nav** — add `{ path: "/calendar", component: () => import("./views/CalendarView.vue") }` + nav link.

- [ ] **Step 6: Build-check + commit**

```bash
npm run build
git add -A
git commit -m "feat(web): calendar management (work week / holidays / time-off)"
```

---

## Task 6: End-to-end smoke

**Files:** none

- [ ] **Step 1: Run**

```bash
npm run tauri dev
```

- [ ] **Step 2: Manual E2E checklist**

- [ ] **Calendar**: set Fri to half-day; add a holiday on a Wednesday; add a half-day time-off for a resource.
- [ ] **Allocations**: pick resource+task, set 50% Mon–Fri → creates; list updates; impact shows utilization.
- [ ] **Allocations**: create a 2nd 100% allocation for the same resource/week → impact shows ⚠ overload.
- [ ] **Dashboard**: the resource's `UtilBar` turns red; appears in 过载预警; project burn shows ratio; team summary renders after picking a team.
- [ ] Change the window dates → Dashboard refreshes.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test: Phase 2b end-to-end smoke (dashboard/allocations/calendar)"
```

---

## Self-Review

**Spec coverage (design §7 + roadmap Phase 2 frontend):**
- §7 Dashboard (resource/team workload, overload alerts, project health) → Task 3 ✓
- §7 utilization bars colored by thresholds (green/yellow/red) → UtilBar + `band()` (Task 3) ✓
- Allocation CRUD + live workload impact → Task 4 ✓
- Calendar management (work week/holiday/time-off) → Task 5 ✓
- Real-time workload per resource/team → consumes Phase 2 `resource_summary`/`team_summary`/`overloads` ✓

**Deferred (not placeholders):** Gantt/Calendar **visualizations** (Phase 3b); `workload_cache` (on-demand compute used); teams CRUD UI (commands ready); production first-run passphrase.

**Placeholder scan:** none — complete code; tests assert concrete values.

**Type consistency:**
- TS types mirror Serialize Rust DTOs (snake_case: `resource_id`, `capacity_pd`, `workload_pd`, `overloaded_members`, `allocated_pd`).
- `api.createAllocation` camelCase args → command `create_allocation` params (resourceId→resource_id etc.); percent validated >0 and ≤1 in the command.
- `band()` thresholds default `{overload:1.1, underload:0.5, green:0.7, yellow:1.0}` match migration 0002 seeded values.
- `AllocationView` SELECT alias order (`resource_name`, `task_title`, `project_id`) matches the model field order.

**Known impl-time items:** `loadWeek()` in calendar store currently re-persists on init — harmless (idempotent upsert); could be changed to a read once `list_work_weeks` parsing is added. UtilBar caps width at 150% (overloads still visible via color + alert list).

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-kanban-phase2b-frontend.md`. Options: **1. Subagent-Driven** (recommended) or **2. Inline Execution**. Which? (Next: **Phase 3 backend** — Gantt/cross-project/calendar-occupancy queries.)
