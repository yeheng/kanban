# HR Kanban — Phase 1b: Frontend + Kanban Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the running Tauri v2 desktop app with a Vue 3 frontend that drives the Phase 1 backend CRUD and renders a drag-and-drop **Kanban** of tasks grouped by status.

**Architecture:** Tauri v2 shell (`src-tauri/`) wrapping the `app` crate; the Vue 3 SPA (Vite + TypeScript) talks to Rust only via typed `invoke` wrappers. State lives in Pinia stores (projects, tasks, catalog, resources, ui). The Kanban uses native HTML5 drag-and-drop (no extra dep) to move cards between status columns, calling `set_task_status`. Tests cover the typed API client and stores via Vitest (pure logic); the Kanban DnD is verified by an end-to-end manual smoke.

**Tech Stack:** Tauri v2, Vite, Vue 3 (`<script setup>` + TS), Pinia, Vue Router, Naive UI (design §7 recommendation), `@tauri-apps/api`, Vitest + `@vue/test-utils`.

**Prerequisite:** Phase 0 + Phase 1 backend plans implemented and `cargo test --workspace` green. The `app` crate exposes `#[tauri::command]` functions (`create_project`, `list_projects`, `ensure_skill`, `list_skills`, `ensure_tag`, `list_tags`, `create_task`, `set_task_status`, `kanban_tasks`, `create_team`, `add_team_member`) and `app::crypto::open_encrypted`.

**Scope note:** Kanban + project/task/resource CRUD only. Gantt, calendar, allocations UI, and the AI panel are later phases. The production first-run passphrase prompt + OS keychain storage (§6.8) is a deferred hardening task — here the DB is opened at startup from an env passphrase with a dev fallback so the app runs end-to-end.

**Reference design:** `docs/design/2026-06-27-hr-kanban-design.md` (§7 Frontend & UI).

---

## File Structure

```
kanban/
├── src-tauri/                      # Tauri v2 shell (NEW)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── icons/
│   └── src/
│       ├── main.rs                 # tauri::Builder + manage(AppState) + generate_handler!
│       └── commands_extra.rs       # create_resource/list_resource commands (repo from Phase 0)
├── index.html                      # NEW
├── package.json                    # NEW
├── vite.config.ts                  # NEW
├── tsconfig.json                   # NEW
├── src/                            # Vue SPA (NEW)
│   ├── main.ts
│   ├── App.vue
│   ├── router.ts
│   ├── api/
│   │   ├── index.ts                # typed invoke wrappers
│   │   └── index.test.ts
│   ├── stores/
│   │   ├── projects.ts
│   │   ├── tasks.ts
│   │   ├── catalog.ts              # skills + tags
│   │   ├── resources.ts
│   │   └── *.test.ts
│   ├── views/
│   │   ├── KanbanView.vue
│   │   ├── ProjectsView.vue
│   │   ├── ResourcesView.vue
│   │   └── FirstRun.vue
│   ├── components/
│   │   ├── AppLayout.vue
│   │   ├── KanbanColumn.vue
│   │   ├── TaskCard.vue
│   │   ├── ProjectForm.vue
│   │   ├── TaskForm.vue
│   │   └── ResourceForm.vue
│   └── types.ts
└── tests/                          # (Rust integration tests stay in crates/*/tests)
```

**Responsibilities:** `src-tauri` is the thin shell — it constructs `AppState` (encrypted open) and registers commands. `src/api` is the ONLY place that calls `invoke`; everything else uses Pinia stores. Views are dumb; stores own data + actions.

---

## Task 1: Scaffold Tauri v2 + Vite + Vue 3 + TS

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src/main.ts`, `src/App.vue`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`

- [ ] **Step 1: `package.json`**

```json
{
  "name": "hr-kanban",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "naive-ui": "^2.38.0",
    "pinia": "^2.2.0",
    "vue": "^3.4.0",
    "vue-router": "^4.4.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.0",
    "@vue/test-utils": "^2.4.0",
    "jsdom": "^24.0.0",
    "typescript": "^5.5.0",
    "vite": "^5.3.0",
    "vitest": "^2.0.0",
    "vue-tsc": "^2.0.0"
  }
}
```

- [ ] **Step 2: `vite.config.ts`**

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  test: { environment: "jsdom", globals: true },
});
```

- [ ] **Step 3: `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "jsx": "preserve",
    "types": ["vitest/globals"],
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

- [ ] **Step 4: `index.html`**

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Development Resource Kanban</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 5: `src/main.ts`**

```ts
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";

createApp(App).use(createPinia()).use(router).mount("#app");
```

- [ ] **Step 6: `src/App.vue` (placeholder; replaced in Task 5)**

```vue
<script setup lang="ts"></script>
<template>
  <div style="font-family: sans-serif; padding: 2rem">HR Kanban — scaffold OK</div>
</template>
```

- [ ] **Step 7: `src-tauri/Cargo.toml`**

```toml
[package]
name = "hr-kanban"
version = "0.1.0"
edition.workspace = true

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
app = { path = "../crates/app" }
db = { path = "../crates/db" }
domain = { path = "../crates/domain" }
tokio = { workspace = true }
serde = { version = "1", features = ["derive"] }
tauri = { version = "2", features = [] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 8: `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 9: `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Development Resource Kanban",
  "version": "0.1.0",
  "identifier": "com.hrkanban.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      { "title": "Development Resource Kanban", "width": 1280, "height": 800 }
    ],
    "security": { "csp": null }
  },
  "bundle": { "active": true, "targets": "all" }
}
```

- [ ] **Step 10: Install + verify the web build**

```bash
npm install
npm run build
```
Expected: `dist/` produced, no TS errors.

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri v2 + Vite + Vue 3 + TS"
```

---

## Task 2: Tauri wiring — AppState, command registration, dev DB open

**Files:**
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/commands_extra.rs`
- Modify: workspace `Cargo.toml` (add `src-tauri` member, excluded from default test if needed)

- [ ] **Step 1: Add resource commands the UI needs — `src-tauri/src/commands_extra.rs`**

(Phase 0's `ResourcesRepo` exists but has no command yet; add two thin wrappers.)

```rust
use app::error::AppError;
use app::state::AppState;
use db::models::Resource;
use db::ResourcesRepo;

#[tauri::command]
pub async fn create_resource(state: tauri::State<'_, AppState>, name: String, email: Option<String>) -> Result<i64, AppError> {
    Ok(ResourcesRepo::create(&state.pool, &name, email.as_deref()).await?)
}

#[tauri::command]
pub async fn list_resources(state: tauri::State<'_, AppState>) -> Result<Vec<Resource>, AppError> {
    Ok(ResourcesRepo::list_active(&state.pool).await?)
}
```

- [ ] **Step 2: `src-tauri/src/main.rs`**

```rust
mod commands_extra;

use app::state::AppState;
use std::env;

// Re-export every Phase 1 command so the handler list stays in one place.
use app::command::{
    add_team_member, create_project, create_task, create_team, ensure_skill, ensure_tag,
    kanban_tasks, list_projects, list_skills, list_tags, set_task_status, set_team_override,
};
use commands_extra::{create_resource, list_resources};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db_path = db_path_for(&handle);
                let state = match env::var("HRK_DB_PASSPHRASE") {
                    Ok(p) if !p.is_empty() => app::crypto::open_encrypted(&db_path, &p)
                        .await
                        .expect("failed to open encrypted DB"),
                    _ => {
                        // Dev fallback: unencrypted file so `tauri dev` runs without a passphrase.
                        // Production first-run prompt + keychain is a deferred task (§6.8).
                        let url = format!("sqlite://{}?mode=rwc", db_path);
                        let pool = db::pool::connect(&url).await.expect("dev db open");
                        sqlx::migrate!("../crates/db/migrations").run(&pool).await.expect("migrate");
                        AppState { pool }
                    }
                };
                handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_project, list_projects,
            ensure_skill, list_skills, ensure_tag, list_tags,
            create_task, set_task_status, kanban_tasks,
            create_team, add_team_member, set_team_override,
            create_resource, list_resources,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn db_path_for(app: &tauri::AppHandle) -> String {
    use tauri::Manager;
    let dir = app.path().app_data_dir().expect("app_data_dir");
    std::fs::create_dir_all(&dir).ok();
    dir.join("hrk.db").to_string_lossy().into_owned()
}
```

> Commands are registered immediately; `AppState` is managed asynchronously inside `setup`. A command invoked before the pool is ready will fail with a Tauri "state not managed" error — acceptable during the brief startup window. (If needed, gate the UI behind the `FirstRun` view until ready; see Task 5.)

- [ ] **Step 3: Add `src-tauri` to the workspace**

Root `Cargo.toml` `members`:
```toml
members = ["crates/domain", "crates/db", "crates/app", "src-tauri"]
```

Add to root `Cargo.toml` `[workspace.dependencies]`:
```toml
tauri = { version = "2", features = [] }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p hr-kanban`
Expected: clean build (may emit Tauri capability warnings — fine).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(tauri): wire AppState + command handler + dev DB open"
```

---

## Task 3: Typed API client (TDD)

**Files:**
- Create: `src/types.ts`
- Create: `src/api/index.ts`
- Create: `src/api/index.test.ts`

- [ ] **Step 1: `src/types.ts`** — mirrors the Serialize-ing Rust models (snake_case keys).

```ts
export interface Project {
  id: number; name: string; description: string | null;
  start_date: string | null; end_date: string | null;
  priority: number; budget_pd: number;
  max_parallel_tasks_per_day: number | null; status: string;
}
export interface KanbanTask {
  id: number; project_id: number; title: string; status: string;
  sort_order: number; estimate_pd: number;
  assignee: string | null; skill_count: number;
}
export interface Skill { id: number; name: string; }
export interface Tag { id: number; name: string; color: string | null; }
export interface Resource { id: number; name: string; email: string | null; status: string; }
export type TaskStatus = "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled";
```

- [ ] **Step 2: `src/api/index.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";
import type { Project, KanbanTask, Skill, Tag, Resource, TaskStatus } from "../types";

// Skill requirement tuple mirrors Rust (i64,i64,bool,f64): [skillId, minProf, mandatory, weight]
export type SkillReq = [number, number, boolean, number];

export const api = {
  listProjects: (): Promise<Project[]> => invoke("list_projects"),
  createProject: (name: string, priority: number, budgetPd: number): Promise<number> =>
    invoke("create_project", { name, priority, budgetPd: budgetPd }),

  listSkills: (): Promise<Skill[]> => invoke("list_skills"),
  ensureSkill: (name: string): Promise<number> => invoke("ensure_skill", { name }),
  listTags: (): Promise<Tag[]> => invoke("list_tags"),
  ensureTag: (name: string, color: string | null): Promise<number> => invoke("ensure_tag", { name, color }),

  createTask: (args: {
    projectId: number; title: string; estimatePd: number;
    start: string | null; end: string | null;
    skillReqs: SkillReq[]; tagIds: number[];
  }): Promise<number> =>
    invoke("create_task", {
      projectId: args.projectId, title: args.title, estimatePd: args.estimatePd,
      start: args.start, end: args.end, skillReqs: args.skillReqs, tagIds: args.tagIds,
      description: null, isLongTerm: false, sortOrder: 0,
    }),
  setTaskStatus: (id: number, status: TaskStatus): Promise<void> =>
    invoke("set_task_status", { id, status }),
  kanbanTasks: (projectId: number): Promise<KanbanTask[]> => invoke("kanban_tasks", { projectId }),

  listResources: (): Promise<Resource[]> => invoke("list_resources"),
  createResource: (name: string, email: string | null): Promise<number> =>
    invoke("create_resource", { name, email }),
};
```

- [ ] **Step 3: `src/api/index.test.ts`** (mocks `invoke`)

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { api } from "./index";

beforeEach(() => vi.mocked(invoke).mockReset());

describe("api client", () => {
  it("createProject passes snake_case budgetPd", async () => {
    vi.mocked(invoke).mockResolvedValue(7);
    const id = await api.createProject("Atlas", 3, 40);
    expect(id).toBe(7);
    expect(invoke).toHaveBeenCalledWith("create_project", { name: "Atlas", priority: 3, budgetPd: 40 });
  });

  it("createTask maps camelCase to the command args", async () => {
    vi.mocked(invoke).mockResolvedValue(1);
    await api.createTask({ projectId: 2, title: "T", estimatePd: 5, start: null, end: null, skillReqs: [[1, 3, true, 1]], tagIds: [9] });
    const args = vi.mocked(invoke).mock.calls[0][1] as Record<string, unknown>;
    expect(args.projectId).toBe(2);
    expect(args.estimatePd).toBe(5);
    expect(args.isLongTerm).toBe(false);
  });

  it("setTaskStatus calls the command", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await api.setTaskStatus(1, "done");
    expect(invoke).toHaveBeenCalledWith("set_task_status", { id: 1, status: "done" });
  });
});
```

- [ ] **Step 4: Run test — verify PASS**

Run: `npm test -- src/api/index.test.ts`
Expected: `3 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(web): typed invoke API client + tests"
```

---

## Task 4: Pinia stores (TDD)

**Files:**
- Create: `src/stores/projects.ts`, `src/stores/tasks.ts`, `src/stores/catalog.ts`, `src/stores/resources.ts`
- Create: `src/stores/tasks.test.ts`

- [ ] **Step 1: `src/stores/projects.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Project } from "../types";

export const useProjectsStore = defineStore("projects", () => {
  const items = ref<Project[]>([]);
  const current = ref<number | null>(null);

  async function load() { items.value = await api.listProjects(); if (!current.value && items.value.length) current.value = items.value[0].id; }
  async function create(name: string, priority: number, budgetPd: number) { await api.createProject(name, priority, budgetPd); await load(); }
  function select(id: number) { current.value = id; }

  return { items, current, load, create, select };
});
```

- [ ] **Step 2: `src/stores/tasks.ts`**

```ts
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "../api";
import type { KanbanTask, TaskStatus } from "../types";

const COLUMNS: TaskStatus[] = ["todo", "in_progress", "blocked", "review", "done"];

export const useTasksStore = defineStore("tasks", () => {
  const tasks = ref<KanbanTask[]>([]);

  async function load(projectId: number) { tasks.value = await api.kanbanTasks(projectId); }
  async function moveStatus(taskId: number, status: TaskStatus) {
    const t = tasks.value.find((x) => x.id === taskId);
    if (!t) return;
    const prev = t.status;
    t.status = status;                       // optimistic
    try { await api.setTaskStatus(taskId, status); }
    catch (e) { t.status = prev; throw e; }  // rollback on failure
  }
  function byStatus(status: TaskStatus): KanbanTask[] {
    return tasks.value.filter((t) => t.status === status).sort((a, b) => a.sort_order - b.sort_order);
  }
  const columns = computed(() => COLUMNS);

  return { tasks, columns, load, moveStatus, byStatus };
});
```

- [ ] **Step 3: `src/stores/catalog.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Skill, Tag } from "../types";

export const useCatalogStore = defineStore("catalog", () => {
  const skills = ref<Skill[]>([]);
  const tags = ref<Tag[]>([]);
  async function load() { [skills.value, tags.value] = await Promise.all([api.listSkills(), api.listTags()]); }
  async function ensureSkill(name: string) { const id = await api.ensureSkill(name); await load(); return id; }
  async function ensureTag(name: string, color: string | null) { const id = await api.ensureTag(name, color); await load(); return id; }
  return { skills, tags, load, ensureSkill, ensureTag };
});
```

- [ ] **Step 4: `src/stores/resources.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { Resource } from "../types";

export const useResourcesStore = defineStore("resources", () => {
  const items = ref<Resource[]>([]);
  async function load() { items.value = await api.listResources(); }
  async function create(name: string, email: string | null) { await api.createResource(name, email); await load(); }
  return { items, load, create };
});
```

- [ ] **Step 5: `src/stores/tasks.test.ts`**

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useTasksStore } from "./tasks";
import { api } from "../api";

vi.mock("../api", () => ({
  api: { kanbanTasks: vi.fn(), setTaskStatus: vi.fn() },
}));

beforeEach(() => { setActivePinia(createPinia()); vi.mocked(api.kanbanTasks).mockReset(); vi.mocked(api.setTaskStatus).mockReset(); });

describe("tasks store", () => {
  it("groups tasks by status", async () => {
    vi.mocked(api.kanbanTasks).mockResolvedValue([
      { id: 1, project_id: 1, title: "A", status: "todo", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
      { id: 2, project_id: 1, title: "B", status: "done", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
    ]);
    const s = useTasksStore();
    await s.load(1);
    expect(s.byStatus("todo").map((t) => t.id)).toEqual([1]);
    expect(s.byStatus("done").map((t) => t.id)).toEqual([2]);
  });

  it("moveStatus updates optimistically and rolls back on error", async () => {
    vi.mocked(api.kanbanTasks).mockResolvedValue([
      { id: 1, project_id: 1, title: "A", status: "todo", sort_order: 0, estimate_pd: 1, assignee: null, skill_count: 0 },
    ]);
    vi.mocked(api.setTaskStatus).mockRejectedValueOnce(new Error("boom"));
    const s = useTasksStore();
    await s.load(1);
    await expect(s.moveStatus(1, "in_progress")).rejects.toThrow("boom");
    expect(s.byStatus("todo")[0].id).toBe(1); // rolled back
  });
});
```

- [ ] **Step 6: Run test — verify PASS**

Run: `npm test -- src/stores/tasks.test.ts`
Expected: `2 passed`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(web): Pinia stores (projects/tasks/catalog/resources) + tests"
```

---

## Task 5: App shell — router, layout, nav, FirstRun

**Files:**
- Create: `src/router.ts`
- Create: `src/components/AppLayout.vue`
- Create: `src/views/FirstRun.vue`
- Modify: `src/App.vue`

- [ ] **Step 1: `src/router.ts`**

```ts
import { createRouter, createWebHashHistory } from "vue-router";
import KanbanView from "./views/KanbanView.vue";
import ProjectsView from "./views/ProjectsView.vue";
import ResourcesView from "./views/ResourcesView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/kanban" },
    { path: "/kanban", component: KanbanView },
    { path: "/projects", component: ProjectsView },
    { path: "/resources", component: ResourcesView },
  ],
});
```

- [ ] **Step 2: `src/components/AppLayout.vue`**

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const ready = ref(false);

onMounted(async () => {
  // Retry briefly until the Rust AppState (managed async in setup) is ready.
  for (let i = 0; i < 40; i++) {
    try { await projects.load(); await catalog.load(); ready.value = true; return; }
    catch { await new Promise((r) => setTimeout(r, 100)); }
  }
});
</script>

<template>
  <n-layout has-sider style="height: 100vh">
    <n-layout-sider bordered content-style="padding:16px" :width="200">
      <h3 style="margin-top:0">HR Kanban</h3>
      <router-link to="/kanban" style="display:block;padding:6px 0">看板 Kanban</router-link>
      <router-link to="/projects" style="display:block;padding:6px 0">项目 Projects</router-link>
      <router-link to="/resources" style="display:block;padding:6px 0">资源 Resources</router-link>
      <hr />
      <small>项目：</small>
      <select v-model.number="projects.current" @change="projects.select(projects.current!)">
        <option v-for="p in projects.items" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </n-layout-sider>
    <n-layout-content content-style="padding:16px">
      <div v-if="!ready">正在打开数据库…</div>
      <router-view v-else />
    </n-layout-content>
  </n-layout>
</template>
```

- [ ] **Step 3: `src/views/FirstRun.vue`** (placeholder for the deferred §6.8 first-run flow)

```vue
<script setup lang="ts"></script>
<template>
  <div style="padding:2rem;font-family:sans-serif">
    <p>设置数据库主口令（首次启用）—— 完整首启流程见 §6.8（后续任务）。</p>
    <p>当前开发模式：以 <code>HRK_DB_PASSPHRASE</code> 环境变量启动加密库，否则使用未加密开发库。</p>
  </div>
</template>
```

- [ ] **Step 4: `src/App.vue`**

```vue
<script setup lang="ts">
import { NConfigProvider } from "naive-ui";
import AppLayout from "./components/AppLayout.vue";
</script>

<template>
  <n-config-provider>
    <AppLayout />
  </n-config-provider>
</template>
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(web): app shell (router/layout/nav) + first-run placeholder"
```

---

## Task 6: Kanban view — columns, cards, drag-and-drop

**Files:**
- Create: `src/components/TaskCard.vue`
- Create: `src/components/KanbanColumn.vue`
- Create: `src/views/KanbanView.vue`

- [ ] **Step 1: `src/components/TaskCard.vue`**

```vue
<script setup lang="ts">
import type { KanbanTask } from "../types";
defineProps<{ task: KanbanTask }>();
const emit = defineEmits<{ (e: "dragstart", id: number): void }>();
</script>

<template>
  <div
    class="card"
    draggable="true"
    @dragstart="emit('dragstart', task.id)"
  >
    <div class="title">{{ task.title }}</div>
    <div class="meta">
      <span>{{ task.estimate_pd }} PD</span>
      <span v-if="task.skill_count">· {{ task.skill_count }} skill(s)</span>
    </div>
    <div v-if="task.assignee" class="assignee">@{{ task.assignee }}</div>
  </div>
</template>

<style scoped>
.card { background:#fff; border:1px solid #e0e0e6; border-radius:6px; padding:8px 10px; margin-bottom:8px; cursor:grab; box-shadow:0 1px 2px rgba(0,0,0,.04); }
.title { font-weight:600; }
.meta { font-size:12px; color:#888; margin-top:4px; }
.assignee { font-size:12px; color:#2080f0; margin-top:2px; }
</style>
```

- [ ] **Step 2: `src/components/KanbanColumn.vue`**

```vue
<script setup lang="ts">
import type { KanbanTask, TaskStatus } from "../types";
import TaskCard from "./TaskCard.vue";
defineProps<{ status: TaskStatus; tasks: KanbanTask[] }>();
const emit = defineEmits<{ (e: "drop", status: TaskStatus): void }>();
const dragging = ref(false);
import { ref } from "vue";
function onDrop() { dragging.value = false; emit("drop", status); }
</script>

<template>
  <div class="column" @dragover.prevent="dragging = true" @dragleave="dragging = false" @drop="onDrop">
    <div class="col-header">{{ status }} ({{ tasks.length }})</div>
    <TaskCard v-for="t in tasks" :key="t.id" :task="t" @dragstart="(id) => $emit('dragstart-card' as any, id)" />
  </div>
</template>

<style scoped>
.column { width:240px; min-width:240px; background:#f5f5f8; border-radius:8px; padding:8px; height:100%; overflow-y:auto; }
.col-header { font-weight:600; text-transform:capitalize; margin-bottom:8px; padding:4px; }
.column:has(.dragging) { outline: none; }
</style>
```

> Note: the column forwards card drag-start via `$emit('dragstart-card', id)`; the parent `KanbanView` tracks the dragged id. (`:has` is a progressive-enhancement nicety; functionally optional.)

- [ ] **Step 3: `src/views/KanbanView.vue`**

```vue
<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import KanbanColumn from "../components/KanbanColumn.vue";
import type { TaskStatus } from "../types";

const tasks = useTasksStore();
const projects = useProjectsStore();
const draggingId = ref<number | null>(null);

watchEffect(async () => {
  if (projects.current) await tasks.load(projects.current);
});

function onDrop(status: TaskStatus) {
  if (draggingId.value == null) return;
  tasks.moveStatus(draggingId.value, status);
  draggingId.value = null;
}
</script>

<template>
  <div>
    <h2 style="margin-top:0">看板 / Kanban</h2>
    <div style="display:flex; gap:12px; align-items:flex-start">
      <KanbanColumn
        v-for="col in tasks.columns"
        :key="col"
        :status="col"
        :tasks="tasks.byStatus(col)"
        @drop="onDrop"
        @dragstart-card="(id: number) => (draggingId = id)"
      />
    </div>
  </div>
</template>
```

- [ ] **Step 4: Build-check**

Run: `npm run build`
Expected: no TS/Vue errors.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(web): Kanban view (status columns + native HTML5 DnD)"
```

---

## Task 7: CRUD forms — Project, Task, Resource

**Files:**
- Create: `src/components/ProjectForm.vue`, `src/components/TaskForm.vue`, `src/components/ResourceForm.vue`
- Create: `src/views/ProjectsView.vue`, `src/views/ResourcesView.vue`

- [ ] **Step 1: `src/components/ProjectForm.vue`**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
const name = ref(""); const priority = ref(5); const budget = ref(0);
async function submit() { if (!name.value.trim()) return; await projects.create(name.value, priority.value, budget.value); name.value = ""; }
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="name" placeholder="项目名" />
    <input v-model.number="priority" type="number" min="1" max="9" />
    <input v-model.number="budget" type="number" min="0" placeholder="budget PD" />
    <button>新建项目</button>
  </form>
</template>
```

- [ ] **Step 2: `src/components/TaskForm.vue`** (picks existing skills/tags from the catalog)

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useProjectsStore } from "../stores/projects";
import { useCatalogStore } from "../stores/catalog";
import { api } from "../api";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const title = ref(""); const estimate = ref(1);
const selectedSkills = ref<number[]>([]);
const selectedTags = ref<number[]>([]);

async function submit() {
  if (!title.value.trim() || !projects.current) return;
  const skillReqs = selectedSkills.value.map((id) => [id, 3, true, 1] as [number, number, boolean, number]);
  await api.createTask({
    projectId: projects.current, title: title.value, estimatePd: estimate.value,
    start: null, end: null, skillReqs, tagIds: selectedTags.value,
  });
  title.value = ""; estimate.value = 1; selectedSkills.value = []; selectedTags.value = [];
  await useTasksStore().load(projects.current);
}
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="title" placeholder="任务标题" />
    <input v-model.number="estimate" type="number" min="0" placeholder="PD" />
    <select v-model="selectedSkills" multiple>
      <option v-for="s in catalog.skills" :key="s.id" :value="s.id">{{ s.name }}</option>
    </select>
    <select v-model="selectedTags" multiple>
      <option v-for="t in catalog.tags" :key="t.id" :value="t.id">{{ t.name }}</option>
    </select>
    <button>新建任务</button>
  </form>
</template>
```

- [ ] **Step 3: `src/components/ResourceForm.vue`**

```vue
<script setup lang="ts">
import { ref } from "vue";
import { useResourcesStore } from "../stores/resources";
const resources = useResourcesStore();
const name = ref(""); const email = ref("");
async function submit() { if (!name.value.trim()) return; await resources.create(name.value, email.value || null); name.value = ""; email.value = ""; }
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="name" placeholder="姓名" />
    <input v-model="email" placeholder="email (可选)" />
    <button>新建资源</button>
  </form>
</template>
```

- [ ] **Step 4: `src/views/ProjectsView.vue`**

```vue
<script setup lang="ts">
import ProjectForm from "../components/ProjectForm.vue";
import TaskForm from "../components/TaskForm.vue";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
</script>
<template>
  <h2>项目 / Projects</h2>
  <ProjectForm />
  <ul>
    <li v-for="p in projects.items" :key="p.id" :style="p.id === projects.current ? 'font-weight:bold' : ''">
      <a href="#" @click.prevent="projects.select(p.id)">{{ p.name }}</a> — 优先级 {{ p.priority }} · 预算 {{ p.budget_pd }} PD
    </li>
  </ul>
  <hr />
  <h3>在当前项目新建任务</h3>
  <TaskForm v-if="projects.current" />
  <p v-else>请先选择一个项目。</p>
</template>
```

- [ ] **Step 5: `src/views/ResourcesView.vue`**

```vue
<script setup lang="ts">
import ResourceForm from "../components/ResourceForm.vue";
import { useResourcesStore } from "../stores/resources";
import { onMounted } from "vue";
const resources = useResourcesStore();
onMounted(() => resources.load());
</script>
<template>
  <h2>资源 / Resources</h2>
  <ResourceForm />
  <ul>
    <li v-for="r in resources.items" :key="r.id">{{ r.name }} <span v-if="r.email">· {{ r.email }}</span></li>
  </ul>
</template>
```

- [ ] **Step 6: Build-check**

Run: `npm run build`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(web): CRUD forms (project/task/resource) + views"
```

---

## Task 8: End-to-end smoke run

**Files:** none (verification only)

- [ ] **Step 1: Run the desktop app (dev mode, unencrypted fallback)**

```bash
npm install        # if not already
cargo build -p hr-kanban
npm run tauri dev
```
Expected: a desktop window opens, sidebar shows, "正在打开数据库…" then the Kanban view.

- [ ] **Step 2: Manual E2E (verify each, check the box)**

- [ ] In **Projects**, create project "Atlas" (priority 3, budget 40) → it appears in the sidebar selector.
- [ ] Select Atlas, create task "Build API" (5 PD, pick a skill) → success.
- [ ] Switch to **Kanban** → the task card appears in the **todo** column with "5 PD · 1 skill".
- [ ] Drag the card to **in_progress** → it moves columns (calls `set_task_status`); refresh-free, persisted (restart app → still in_progress).
- [ ] Create a resource in **Resources** → appears in list.

- [ ] **Step 3: Run encrypted (verify Phase 1 Task 8 wiring end-to-end)**

```bash
rm -rf ~/Library/Application\ Support/com.hrkanban.app   # clear dev db
HRK_DB_PASSPHRASE="correct-horse-battery-staple" npm run tauri dev
```
Expected: app opens encrypted DB (created on first run), same CRUD works; the DB file at the app-data dir is encrypted (not readable as plain SQLite).

- [ ] **Step 4: Commit the working state**

```bash
git add -A
git commit -m "test: Phase 1b end-to-end smoke (kanban DnD + encrypted open)"
```

---

## Self-Review

**Spec coverage (design §7 frontend + roadmap Phase 1):**
- §7 Naive UI + Pinia + Vue Router selection → Tasks 1, 4, 5 ✓
- §7 typed invoke interaction layer → Task 3 ✓
- §7 Kanban view (status columns, cards w/ assignee + skills) → Task 6 ✓
- §7 Kanban DnD → status change → Task 6 (moveStatus → set_task_status) ✓
- §7 resource/project/task CRUD UIs → Task 7 ✓
- §6.8 encrypted open (confirmed #55) → Task 2 wiring + Task 8 encrypted smoke ✓
- Roadmap "资源/团队/项目/任务 CRUD + Kanban" → Kanban (T6) + project/task/resource CRUD (T7); **teams CRUD UI deferred** (commands exist in backend; a TeamsView following the ResourcesView pattern is a trivial follow-up) — noted.

**Deferred (explicitly out of scope, not placeholders):**
- Gantt, calendar, allocations UI, Dashboard/workload, AI panel → later phases.
- Production first-run passphrase prompt + OS keychain storage (§6.8) → deferred task (Task 5 `FirstRun.vue` is a placeholder; full flow needs lazy-unlock or keychain read at startup).
- Teams CRUD view (backend commands ready) → trivial follow-up.
- Component/visual testing (Playwright) → the Kanban DnD is verified by the Task 8 manual smoke; stores + API are unit-tested.

**Placeholder scan:** none. Every code step contains complete code; tests assert concrete values.

**Type consistency:**
- `api.createProject(name, priority, budgetPd)` → command args `{name, priority, budgetPd}` match Rust `create_project(name, description?, start?, end?, priority, budget_pd?)` — `budgetPd` maps to `budget_pd` via Tauri's snake_case command-arg convention (Tauri v2 converts camelCase JS keys to snake_case Rust params). Test asserts this.
- `api.createTask` camelCase args map to `create_task` Rust params (`projectId→project_id`, `estimatePd→estimate_pd`, `isLongTerm→is_long_term`, `skillReqs→skill_reqs`, `tagIds→tag_ids`).
- `KanbanTask` TS fields match the Rust `KanbanTask` Serialize fields (snake_case: `project_id`, `sort_order`, `estimate_pd`, `skill_count`).
- `TaskStatus` TS union matches the Rust `tasks.status` CHECK constraint values.

**Known impl-time items (from design, not blockers):**
- Tauri v2 converts JS camelCase invoke arg keys to Rust snake_case params automatically; verify with the Task 3 test (if a command reports a missing arg, align the key).
- AppState is managed async in `setup`; `AppLayout` polls until ready (40 × 100 ms). If startup is slower, raise the retry count or surface an error.
- The Kanban uses native HTML5 DnD (no library); touch-device support is out of scope (desktop app).

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-27-hr-kanban-phase1b-frontend.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks.
2. **Inline Execution** — Execute tasks in this session in batches with checkpoints.

Which approach? (Phase 0 → Phase 1 backend → **Phase 1b** now form a runnable desktop app with Kanban. Next would be Phase 2: allocations UI + Dashboard/workload, or the deferred teams CRUD / first-run hardening.)
