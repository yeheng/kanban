# HR Kanban — Phase 4b: Frontend (AI Panel) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the **AI optimization panel** — run optimization for a project, review the proposed plan (assignments, unscheduled, metrics, LLM/template explanation), tune multi-objective weights, and accept/reject (human-in-the-loop), plus a run history list.

**Architecture:** One small backend tweak (`run_optimization` accepts optional `ObjectiveWeights`, confirmed #6). Frontend adds a typed API, an `optimization` Pinia store, and an `AiPanelView`. The proposed plan renders as an assignments table + metrics bars + the explanation text (plain preformatted — no markdown dep; `marked` optional). Tests cover the store via Vitest.

**Tech Stack:** Vue 3 + TS, Pinia, Naive UI, Tauri invoke, Vitest. No new deps.

**Prerequisite:** Phase 4 AI engine green (`run_optimization`, `list_optimization_runs`, `apply_solution`, `reject_solution` commands; `RunResult`/`RunRow` types).

**Scope note:** Run / review / accept / reject / history + weight tuning. What-if multi-scenario overlay (confirmed #36) and Gantt overlay preview are later refinements (design §7.5.6; needs Phase 3 Gantt).

**Reference design:** `docs/design/2026-06-27-hr-kanban-design.md` (§7.5.6 AI panel, §5 ObjectiveWeights).

---

## File Structure

```
kanban/
├── crates/app/src/
│   ├── command.rs               # MOD: run_optimization accepts Option<ObjectiveWeights>
│   └── service/optimization.rs  # MOD: run_for_project(.., weights)
└── src/
    ├── api/index.ts             # MOD: runOptimization/listRuns/apply/reject + weights
    ├── types.ts                 # MOD: RunResult/RunRow/ObjectiveWeights/ScoredAssignment
    ├── stores/optimization.ts   # NEW (+ test)
    ├── components/
    │   ├── WeightsPanel.vue     # NEW (skill_fit/balance/budget sliders)
    │   └── PlanReview.vue       # NEW (assignments table + metrics + explanation)
    └── views/AiPanelView.vue    # NEW
```

---

## Task 1: `run_optimization` weights param + API + store (TDD)

**Files:**
- Modify: `crates/app/src/service/optimization.rs`, `crates/app/src/command.rs`
- Modify: `src/types.ts`, `src/api/index.ts`
- Create: `src/stores/optimization.ts` (+ test)

- [ ] **Step 1: Backend — accept optional weights**

In `crates/app/src/service/optimization.rs`, change `run_for_project` signature and apply the override:

```rust
pub async fn run_for_project(
    pool: &SqlitePool, project_id: i64, weights: Option<ai_engine::ObjectiveWeights>,
) -> Result<RunResult, AppError> {
    let mut problem = build_problem(pool, project_id).await?;
    if let Some(w) = weights { problem.weights = w; }
    // ... (rest unchanged: engine.optimize, persist, return)
```

In `crates/app/src/command.rs`:

```rust
use ai_engine::ObjectiveWeights;

#[tauri::command]
pub async fn run_optimization(
    state: tauri::State<'_, AppState>, project_id: i64, weights: Option<ObjectiveWeights>,
) -> Result<RunResult, AppError> {
    OptimizationService::run_for_project(&state.pool, project_id, weights).await
}
```

- [ ] **Step 2: Types — append to `src/types.ts`**

```ts
export interface ObjectiveWeights { skill_fit: number; balance: number; budget: number; }
export interface ScoredAssignment {
  resource_id: number; task_id: number; start: string; end: string;
  percent: number; score: number; rationale: string;
}
export interface SolutionMetrics { overall: number; skill_fit: number; utilization: number; fairness: number; }
export interface Solution { run_id: number; assignments: ScoredAssignment[]; unscheduled: number[]; metrics: SolutionMetrics; }
export interface RunResult { run_id: number; plan: { solution: Solution; explanation_md: string; } }
export interface RunRow { id: number; objective: string; status: string; applied: number; score_overall: number | null; created_at: string; }
```

- [ ] **Step 3: API — append to `src/api/index.ts`**

```ts
  runOptimization: (projectId: number, weights: ObjectiveWeights | null) =>
    invoke<RunResult>("run_optimization", { projectId, weights }),
  listOptimizationRuns: (limit: number | null) => invoke<RunRow[]>("list_optimization_runs", { limit }),
  applySolution: (runId: number) => invoke<number>("apply_solution", { runId }),
  rejectSolution: (runId: number) => invoke<void>("reject_solution", { runId }),
```

- [ ] **Step 4: `src/stores/optimization.ts`**

```ts
import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "../api";
import type { RunResult, RunRow, ObjectiveWeights } from "../types";

export const useOptimizationStore = defineStore("optimization", () => {
  const current = ref<RunResult | null>(null);
  const history = ref<RunRow[]>([]);
  const weights = ref<ObjectiveWeights>({ skill_fit: 0.4, balance: 0.4, budget: 0.2 });
  const busy = ref(false);

  async function run(projectId: number) {
    busy.value = true;
    try { current.value = await api.runOptimization(projectId, weights.value); }
    finally { busy.value = false; }
  }
  async function loadHistory() { history.value = await api.listOptimizationRuns(20); }
  async function accept(runId: number) { await api.applySolution(runId); current.value = null; await loadHistory(); }
  async function reject(runId: number) { await api.rejectSolution(runId); current.value = null; await loadHistory(); }
  function normalize() {
    const s = weights.value.skill_fit + weights.value.balance + weights.value.budget;
    if (s > 0) weights.value = {
      skill_fit: weights.value.skill_fit / s,
      balance: weights.value.balance / s,
      budget: weights.value.budget / s,
    };
  }
  return { current, history, weights, busy, run, loadHistory, accept, reject, normalize };
});
```

- [ ] **Step 5: `src/stores/optimization.test.ts`**

```ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useOptimizationStore } from "./optimization";
import { api } from "../api";

vi.mock("../api", () => ({ api: { runOptimization: vi.fn(), listOptimizationRuns: vi.fn(), applySolution: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); });

describe("optimization store", () => {
  it("runs with current weights and stores plan", async () => {
    vi.mocked(api.runOptimization).mockResolvedValue({ run_id: 7, plan: { solution: { run_id: 7, assignments: [], unscheduled: [], metrics: { overall: 80, skill_fit: 80, utilization: 100, fairness: 0 } }, explanation_md: "ok" } });
    const s = useOptimizationStore();
    await s.run(5);
    expect(s.current?.run_id).toBe(7);
    expect(vi.mocked(api.runOptimization)).toHaveBeenCalledWith(5, s.weights);
  });
  it("normalize makes weights sum to 1", () => {
    const s = useOptimizationStore();
    s.weights = { skill_fit: 1, balance: 1, budget: 2 };
    s.normalize();
    const sum = s.weights.skill_fit + s.weights.balance + s.weights.budget;
    expect(Math.abs(sum - 1)).toBeLessThan(1e-9);
  });
});
```

- [ ] **Step 6: Build + test + commit**

```bash
cargo build --workspace && npm test -- src/stores/optimization.test.ts
git add -A && git commit -m "feat(app,web): run_optimization weights + optimization store"
```

---

## Task 2: PlanReview + WeightsPanel + AI panel view

**Files:**
- Create: `src/components/WeightsPanel.vue`, `src/components/PlanReview.vue`, `src/views/AiPanelView.vue`
- Modify: `src/router.ts`, nav

- [ ] **Step 1: `src/components/WeightsPanel.vue`**

```vue
<script setup lang="ts">
import { useOptimizationStore } from "../stores/optimization";
const opt = useOptimizationStore();
const labels: (keyof typeof opt.weights)[] = ["skill_fit", "balance", "budget"];
const cn = { skill_fit: "技能最优", balance: "负载均衡", budget: "预算" };
</script>
<template>
  <div class="weights">
    <div v-for="k in labels" :key="k">
      <small>{{ cn[k] }}</small>
      <input type="range" min="0" max="1" step="0.05" v-model.number="opt.weights[k]" @change="opt.normalize()" />
      {{ Math.round(opt.weights[k] * 100) }}
    </div>
  </div>
</template>
<style scoped>
.weights div { display: flex; align-items: center; gap: 8px; margin: 4px 0; font-size: 13px; }
small { width: 80px; }
</style>
```

- [ ] **Step 2: `src/components/PlanReview.vue`**

```vue
<script setup lang="ts">
import { useOptimizationStore } from "../stores/optimization";
const opt = useOptimizationStore();
function pct(v: number) { return Math.round(v) + "%"; }
</script>
<template>
  <div v-if="opt.current">
    <h3>方案 #{{ opt.current.run_id }}</h3>
    <div>综合评分 <b>{{ pct(opt.current.plan.solution.metrics.overall) }}</b> · 技能 {{ pct(opt.current.plan.solution.metrics.skill_fit) }} · 排期覆盖 {{ pct(opt.current.plan.solution.metrics.utilization) }}</div>

    <h4>已分配 ({{ opt.current.plan.solution.assignments.length }})</h4>
    <table border="1" cellpadding="4" style="border-collapse:collapse">
      <tr><th>资源</th><th>任务</th><th>区间</th><th>投入</th><th>匹配分</th></tr>
      <tr v-for="a in opt.current.plan.solution.assignments" :key="a.task_id">
        <td>#{{ a.resource_id }}</td><td>#{{ a.task_id }}</td>
        <td>{{ a.start }} → {{ a.end }}</td><td>{{ Math.round(a.percent * 100) }}%</td>
        <td>{{ Math.round(a.score * 100) }}</td>
      </tr>
    </table>

    <p v-if="opt.current.plan.solution.unscheduled.length" style="color:#d03050">
      ⚠ 未排期任务：{{ opt.current.plan.solution.unscheduled.join(", ") }}
    </p>

    <h4>解释</h4>
    <pre style="white-space:pre-wrap;background:#f7f7fa;padding:8px;border-radius:4px">{{ opt.current.plan.explanation_md }}</pre>

    <div style="margin-top:8px">
      <button @click="opt.accept(opt.current!.run_id)">✓ 采纳（写入分配）</button>
      <button @click="opt.reject(opt.current!.run_id)">✗ 拒绝</button>
    </div>
  </div>
</template>
```

- [ ] **Step 3: `src/views/AiPanelView.vue`**

```vue
<script setup lang="ts">
import { onMounted } from "vue";
import { useOptimizationStore } from "../stores/optimization";
import { useProjectsStore } from "../stores/projects";
import WeightsPanel from "../components/WeightsPanel.vue";
import PlanReview from "../components/PlanReview.vue";

const opt = useOptimizationStore();
const projects = useProjectsStore();
onMounted(() => opt.loadHistory());
</script>
<template>
  <h2 style="margin-top:0">AI 优化 / Optimization</h2>
  <div style="display:flex;gap:24px;align-items:flex-start">
    <div>
      <h3>目标权重</h3>
      <WeightsPanel />
      <button :disabled="!projects.current || opt.busy" @click="opt.run(projects.current!)">
        {{ opt.busy ? "求解中…" : "为当前项目运行优化" }}
      </button>
    </div>
    <div style="flex:1">
      <PlanReview v-if="opt.current" />
      <p v-else style="color:#888">运行优化后在此查看建议方案。</p>
    </div>
  </div>

  <h3>历史运行</h3>
  <table border="1" cellpadding="4" style="border-collapse:collapse">
    <tr><th>#</th><th>状态</th><th>评分</th><th>已采纳</th><th>时间</th></tr>
    <tr v-for="r in opt.history" :key="r.id">
      <td>{{ r.id }}</td><td>{{ r.status }}</td>
      <td>{{ r.score_overall?.toFixed(0) ?? "-" }}</td>
      <td>{{ r.applied ? "是" : "否" }}</td><td>{{ r.created_at }}</td>
    </tr>
  </table>
</template>
```

- [ ] **Step 4: Route + nav** — add `{ path: "/ai", component: () => import("./views/AiPanelView.vue") }` + nav link.

- [ ] **Step 5: Build + commit**

```bash
npm run build && git add -A && git commit -m "feat(web): AI panel (run/review/weights/accept/reject/history)"
```

---

## Task 3: End-to-end smoke

- [ ] **Step 1:** `npm run tauri dev`.

- [ ] **Step 2: Manual checklist**
- [ ] With a project that has tasks + a skilled resource, open **AI 优化** → adjust weight sliders (normalize) → click 运行优化.
- [ ] Proposed plan appears: assignments table, metrics, explanation (TemplateExplainer text), any unscheduled tasks flagged.
- [ ] Click ✓ 采纳 → allocations appear (verify on Kanban/Gantt as `source=ai`); run shows 已采纳=是 in history.
- [ ] Run again → 拒绝 → status becomes rejected; no allocations written.

- [ ] **Step 3:** `git add -A && git commit -m "test: Phase 4b end-to-end smoke (AI panel)"`

---

## Self-Review

**Spec coverage (§7.5.6 + §5 + roadmap Phase 4b):**
- Run optimization + review proposed plan (assignments/unscheduled/metrics/explanation) → Task 2 ✓
- Accept (apply→allocations) / Reject (human-in-the-loop) → Tasks 1–2 ✓
- Multi-objective weights UI-tunable (confirmed #6, default 0.4/0.4/0.2, normalized) → WeightsPanel (Task 2) + backend param (Task 1) ✓
- Run history → Task 2 ✓

**Deferred (not placeholders):** What-if multi-scenario compare + Gantt overlay (confirmed #36; needs Phase 3 Gantt); `marked` for rich markdown (plain `<pre>` now); LLM explanation vs template (engine picks impl; UI agnostic).

**Placeholder scan:** none — complete code; store test asserts run uses current weights + normalize sums to 1.

**Type consistency:** `RunResult`/`RunRow`/`ObjectiveWeights`/`ScoredAssignment` TS mirror Rust Serialize shapes (snake_case: `skill_fit`, `run_id`, `score_overall`); `runOptimization(projectId, weights)` → `run_optimization(project_id, weights)`.

**Known impl-time items:** explanation renders as preformatted text (no markdown rendering dep — add `marked` if rich formatting wanted); weights normalize on change.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-hr-kanban-phase4b-ai-panel.md`. **1. Subagent-Driven** (recommended) or **2. Inline**. Next: **Phase 5 (reports)**.
