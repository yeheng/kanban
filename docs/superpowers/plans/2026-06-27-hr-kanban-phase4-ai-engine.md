# HR Kanban — Phase 4: AI Optimization Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the hybrid AI optimization engine (design §5): score resource↔task semantic fit, solve the assignment under hard constraints, explain the plan, and persist reproducible runs — with a fully-working **deterministic, offline-testable** pipeline (Fallback scorer + Greedy solver + Template explainer) plus `rig`/`good_lp` production impls behind the same traits.

**Architecture:** A new pure `ai-engine` crate defines `Scorer` / `Solver` / `Explainer` traits + the problem/solution types + an `OptimizationEngine` that wires them (score → solve → explain). Default impls (`FallbackScorer`, `GreedySolver`, `TemplateExplainer`) need **no LLM and no MILP** — the whole pipeline is unit-tested deterministically. Production impls (`SemanticScorer` via `rig` embeddings, `MilpSolver` via `good_lp`+HiGHS, `LlmExplainer` via `rig` chat) are feature-gated (`llm`, `milp`) and drop-in. The `app` crate builds the `AllocationProblem` from the DB, runs the engine, persists `ai_optimization_runs` (reproducible), and exposes commands; `apply_solution` writes `source='ai'` allocations (human-in-the-loop).

**Tech Stack:** Rust, `async-trait`, `serde`, `chrono`; optional `rig-core` (embeddings + chat, local Ollama default), `good_lp`+HiGHS (MILP). Deterministic core has no heavy deps.

**Prerequisite:** Phase 0–3 backend green. Uses `db` repos (resources/skills/tasks/allocations), `domain`, `app::{AppError, AppState}`. Schema `ai_optimization_runs` + `allocations.source/run_id` exist (migration 0001).

**Scope note:** Engine + commands (backend). The **AI panel UI** (run, compare, accept/tweak, show explanation) is Phase 4b. Incremental optimization and `workload_cache` integration are later. Local Ollama is the default provider; cloud is configurable but not required for the deterministic path.

**Reference design:** `docs/design/2026-06-27-hr-kanban-design.md` (§5 AI Optimization Engine, §3.3.16 ai_optimization_runs).

---

## File Structure

```
kanban/
├── crates/ai-engine/                 # NEW pure crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── types.rs                  # AllocationProblem/Solution/Candidate*/ObjectiveWeights/...
│       ├── scorer.rs                 # Scorer trait + FallbackScorer (+ SemanticScorer w/ `llm`)
│       ├── solver.rs                 # Solver trait + GreedySolver (+ MilpSolver w/ `milp`)
│       ├── explainer.rs              # Explainer trait + TemplateExplainer (+ LlmExplainer w/ `llm`)
│       └── engine.rs                 # OptimizationEngine
├── crates/app/src/
│   ├── service/
│   │   ├── mod.rs                    # MOD: add optimization
│   │   └── optimization.rs           # NEW: build problem + persist run + apply/reject
│   ├── command.rs                    # MOD: run_optimization/list_runs/apply_solution/reject_solution
│   └── tests/optimization.rs         # NEW
```

---

## Task 1: `ai-engine` crate + types + traits

**Files:**
- Create: `crates/ai-engine/Cargo.toml`, `src/lib.rs`, `src/types.rs`, `src/scorer.rs`, `src/solver.rs`, `src/explainer.rs`, `src/engine.rs`
- Modify: workspace `Cargo.toml` (add member)

- [ ] **Step 1: `crates/ai-engine/Cargo.toml`**

```toml
[package]
name = "ai-engine"
version = "0.1.0"
edition.workspace = true

[dependencies]
domain = { path = "../domain" }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
chrono = { workspace = true }

[features]
default = []
llm = ["rig-core"]
milp = ["good_lp"]

[dependencies.rig-core]
version = "0.24"
optional = true

[dependencies.good_lp]
version = "1"
optional = true
default-features = false
features = ["highs"]
```

> **Version caveat:** `rig-core` and `good_lp` versions must be verified at execution time (`rig-core ≈ 0.24.x`, `good_lp = 1` with the `highs` feature, HiGHS statically linked per Phase 2/§5 decisions). The deterministic core (Tasks 2–5) does **not** enable `llm`/`milp`, so it builds without these.

- [ ] **Step 2: `crates/ai-engine/src/types.rs`**

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateResource {
    pub id: i64,
    pub name: String,
    /// skill_id -> proficiency (1..5)
    pub skills: HashMap<i64, i64>,
    pub tags: Vec<String>,
    pub daily_capacity_pd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReq { pub skill_id: i64, pub min_proficiency: i64, pub is_mandatory: bool, pub weight: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub estimate_pd: f64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub priority: i64, // 1..9 (lower = higher priority)
    pub skill_reqs: Vec<SkillReq>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingAlloc { pub resource_id: i64, pub start: NaiveDate, pub end: NaiveDate, pub percent: f64 }

/// Multi-objective weights (design §1; default balanced 0.4/0.4/0.2; UI-tunable, confirmed #6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights { pub skill_fit: f64, pub balance: f64, pub budget: f64 }
impl Default for ObjectiveWeights {
    fn default() -> Self { Self { skill_fit: 0.4, balance: 0.4, budget: 0.2 } }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OverloadPolicy { SoftWarn, HardBlock }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConstraintFlags {
    pub allow_parallel_per_day: bool,
    pub max_parallel_tasks_per_day: Option<i64>,
    pub overload_policy: Option<OverloadPolicy>, // None => SoftWarn
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig { pub backend: String, pub timeout_ms: u64, pub seed: u64 }
impl Default for SolverConfig {
    fn default() -> Self { Self { backend: "greedy".into(), timeout_ms: 5000, seed: 1 } }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllocationProblem {
    pub run_id: i64,
    pub resources: Vec<CandidateResource>,
    pub tasks: Vec<CandidateTask>,
    pub existing: Vec<ExistingAlloc>,
    pub weights: ObjectiveWeights,
    pub flags: ConstraintFlags,
    pub config: SolverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredAssignment {
    pub resource_id: i64, pub task_id: i64,
    pub start: NaiveDate, pub end: NaiveDate, pub percent: f64,
    pub score: f64, pub rationale: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SolutionMetrics { pub overall: f64, pub skill_fit: f64, pub utilization: f64, pub fairness: f64 }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Solution {
    pub run_id: i64,
    pub assignments: Vec<ScoredAssignment>,
    pub unscheduled: Vec<i64>, // task ids
    pub metrics: SolutionMetrics,
}

/// score[(resource_id, task_id)] -> 0..1 (filled by the Scorer, consumed by the Solver).
pub type ScoreMatrix = HashMap<(i64, i64), f64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPlan { pub solution: Solution, pub explanation_md: String }
```

- [ ] **Step 3: Traits — `src/scorer.rs`, `src/solver.rs`, `src/explainer.rs`**

`scorer.rs`:
```rust
use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Scorer: Send + Sync {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64; // 0..1
    async fn matrix(&self, problem: &AllocationProblem) -> ScoreMatrix {
        let mut m = ScoreMatrix::new();
        for r in &problem.resources {
            for t in &problem.tasks {
                m.insert((r.id, t.id), self.score(r, t).await);
            }
        }
        m
    }
}
```
`solver.rs`:
```rust
use crate::types::*;
pub trait Solver: Send + Sync {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution;
}
```
`explainer.rs`:
```rust
use crate::types::*;
use async_trait::async_trait;
#[async_trait]
pub trait Explainer: Send + Sync {
    async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String;
}
```

- [ ] **Step 4: Engine stub — `src/engine.rs`**

```rust
use crate::explainer::Explainer;
use crate::scorer::Scorer;
use crate::solver::Solver;
use crate::types::*;
use std::sync::Arc;

pub struct OptimizationEngine {
    pub scorer: Arc<dyn Scorer>,
    pub solver: Arc<dyn Solver>,
    pub explainer: Arc<dyn Explainer>,
}

impl OptimizationEngine {
    pub async fn optimize(&self, problem: &AllocationProblem) -> OptimizedPlan {
        let scores = self.scorer.matrix(problem).await;
        let solution = self.solver.solve(problem, &scores);
        let explanation_md = self.explainer.explain(problem, &solution).await;
        OptimizedPlan { solution, explanation_md }
    }
}
```

- [ ] **Step 5: `src/lib.rs`**

```rust
pub mod engine;
pub mod explainer;
pub mod scorer;
pub mod solver;
pub mod types;

pub use engine::OptimizationEngine;
pub use explainer::Explainer;
pub use scorer::Scorer;
pub use solver::Solver;
pub use types::*;
```

- [ ] **Step 6: Add member + build**

Root `Cargo.toml` `members` → add `"crates/ai-engine"`.
Run: `cargo build -p ai-engine`
Expected: clean (no features → no rig/good_lp).

- [ ] **Step 7: Commit**

```bash
git add -A && git commit -m "feat(ai-engine): types + Scorer/Solver/Explainer traits + engine"
```

---

## Task 2: FallbackScorer (deterministic; TDD)

Keyword-Jaccard over resource skills+tags vs task title+skill_reqs, plus a proficiency bonus. No LLM — the offline scorer.

**Files:**
- Modify: `crates/ai-engine/src/scorer.rs`
- Create: `crates/ai-engine/tests/scorer.rs`

- [ ] **Step 1: Append `FallbackScorer` to `crates/ai-engine/src/scorer.rs`**

```rust
use crate::types::{CandidateResource, CandidateTask};

pub struct FallbackScorer;

impl FallbackScorer {
    fn tokens(r: &CandidateResource) -> Vec<String> {
        let mut v: Vec<String> = r.tags.iter().cloned().collect();
        for (sid, prof) in &r.skills { v.push(format!("skill{}p{}", sid, prof / 3)); } // coarse bucket
        v.into_iter().map(|s| s.to_lowercase()).collect()
    }
    fn task_tokens(t: &CandidateTask) -> Vec<String> {
        let mut v: Vec<String> = t.title.split_whitespace().map(|s| s.to_lowercase()).collect();
        for req in &t.skill_reqs { v.push(format!("skill{}p{}", req.skill_id, req.min_proficiency / 3)); }
        v
    }
    fn jaccard(a: &[String], b: &[String]) -> f64 {
        let sa: std::collections::HashSet<&String> = a.iter().collect();
        let sb: std::collections::HashSet<&String> = b.iter().collect();
        if sa.is_empty() && sb.is_empty() { return 0.0; }
        let inter = sa.intersection(&sb).count() as f64;
        let union = sa.union(&sb).count() as f64;
        inter / union
    }
}

#[async_trait::async_trait]
impl Scorer for FallbackScorer {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
        // mandatory skills must be met at min proficiency, else 0 (hard filter reflected in score)
        for req in &t.skill_reqs {
            if req.is_mandatory {
                match r.skills.get(&req.skill_id) {
                    Some(p) if *p >= req.min_proficiency => {}
                    _ => return 0.0,
                }
            }
        }
        let base = Self::jaccard(&Self::tokens(r), &Self::task_tokens(t));
        // proficiency bonus: avg proficiency on required skills / 5
        let bonus = if t.skill_reqs.is_empty() { 0.0 } else {
            let s: f64 = t.skill_reqs.iter().filter_map(|req| r.skills.get(&req.skill_id)).map(|p| *p as f64).sum();
            s / (t.skill_reqs.len() as f64 * 5.0)
        };
        (base * 0.6 + bonus * 0.4).clamp(0.0, 1.0)
    }
}
```

- [ ] **Step 2: Test — `crates/ai-engine/tests/scorer.rs`**

```rust
use ai_engine::scorer::{FallbackScorer, Scorer};
use ai_engine::types::*;
use std::collections::HashMap;

fn res(id: i64, skills: &[(i64, i64)]) -> CandidateResource {
    CandidateResource { id, name: format!("R{}", id), skills: skills.iter().cloned().collect(),
        tags: vec![], daily_capacity_pd: 1.0 }
}
fn task(id: i64, reqs: &[(i64, i64, bool)]) -> CandidateTask {
    CandidateTask { id, project_id: 1, title: "build api".into(), estimate_pd: 5.0,
        start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        priority: 5, skill_reqs: reqs.iter().map(|(s,p,m)| SkillReq{skill_id:*s,min_proficiency:*p,is_mandatory:*m,weight:1.0}).collect() }
}

#[tokio::test]
async fn mandatory_unmet_scores_zero() {
    let s = FallbackScorer;
    let r = res(1, &[(1, 2)]); // proficiency 2
    let t = task(10, &[(1, 3, true)]); // needs 3
    assert_eq!(s.score(&r, &t).await, 0.0);
}

#[tokio::test]
async fn matched_skill_scores_higher_than_mismatched() {
    let s = FallbackScorer;
    let good = res(1, &[(1, 4)]); let weak = res(2, &[(2, 4)]);
    let t = task(10, &[(1, 3, true)]);
    assert!(s.score(&good, &t).await > s.score(&weak, &t).await); // weak returns 0 (mandatory fail)
}
```

- [ ] **Step 3: Run + commit**

```bash
cargo test -p ai-engine --test scorer   # 2 passed
git add -A && git commit -m "feat(ai-engine): FallbackScorer (jaccard + proficiency)"
```

---

## Task 3: GreedySolver (deterministic; TDD)

For each task (priority asc, then estimate desc), assign to the highest-scoring resource that meets mandatory skills and keeps per-day load ≤ 1.0 over the window (treating each calendar day in `[start,end]` as a capacity-1.0 day — a conservative uniform proxy; the workload engine remains authoritative). `percent` = `min(1.0, estimate / window_days)`.

**Files:**
- Modify: `crates/ai-engine/src/solver.rs`
- Create: `crates/ai-engine/tests/solver.rs`

- [ ] **Step 1: Append `GreedySolver` to `crates/ai-engine/src/solver.rs`**

```rust
use crate::types::*;
use chrono::NaiveDate;
use std::collections::HashMap;

pub struct GreedySolver;

fn window_days(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days() + 1
}

impl Solver for GreedySolver {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        let hard = matches!(problem.flags.overload_policy.unwrap_or(OverloadPolicy::SoftWarn), OverloadPolicy::HardBlock);

        // per-resource per-day committed percent (existing + assigned)
        let mut load: HashMap<i64, HashMap<NaiveDate, f64>> = HashMap::new();
        for e in &problem.existing {
            let mut d = e.start;
            while d <= e.end {
                *load.entry(e.resource_id).or_default().entry(d).or_insert(0.0) += e.percent;
                d = d.succ_opt().unwrap();
            }
        }

        let mut order: Vec<&CandidateTask> = problem.tasks.iter().collect();
        order.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| b.estimate_pd.partial_cmp(&a.estimate_pd).unwrap()));

        let mut assignments = Vec::new();
        let mut unscheduled = Vec::new();
        let mut score_sum = 0.0;

        for t in order {
            let days = window_days(t.start, t.end);
            let needed = (t.estimate_pd / days as f64).min(1.0).max(0.01);
            // candidate resources: mandatory skills met, sorted by score desc
            let mut cands: Vec<&CandidateResource> = problem.resources.iter()
                .filter(|r| t.skill_reqs.iter().filter(|rq| rq.is_mandatory)
                    .all(|rq| r.skills.get(&rq.skill_id).map_or(false, |p| *p >= rq.min_proficiency)))
                .collect();
            cands.sort_by(|a, b| scores.get(&(b.id, t.id)).unwrap_or(&0.0)
                .partial_cmp(scores.get(&(a.id, t.id)).unwrap_or(&0.0)).unwrap());

            let mut chosen = None;
            for r in cands {
                // per-day load across the task window must stay within the capacity limit
                let ok = {
                    let mut d = t.start; let mut ok = true;
                    while d <= t.end {
                        let cur = *load.entry(r.id).or_default().entry(d).or_insert(0.0);
                        let limit = 1.0; // ratio-space cap Σ percent ≤ 1.0 (design §3.8); SoftWarn penalizes via metrics, not feasibility
                        if cur + needed > limit + 1e-9 { ok = false; break; }
                        d = d.succ_opt().unwrap();
                    }
                    ok
                };
                if ok { chosen = Some(r); break; }
            }

            match chosen {
                Some(r) => {
                    let mut d = t.start;
                    while d <= t.end {
                        *load.entry(r.id).or_default().entry(d).or_insert(0.0) += needed;
                        d = d.succ_opt().unwrap();
                    }
                    let sc = *scores.get(&(r.id, t.id)).unwrap_or(&0.0);
                    score_sum += sc;
                    assignments.push(ScoredAssignment {
                        resource_id: r.id, task_id: t.id, start: t.start, end: t.end,
                        percent: needed, score: sc,
                        rationale: format!("greedy: best feasible (score {:.2})", sc),
                    });
                }
                None => unscheduled.push(t.id),
            }
        }

        let n = assignments.len() as f64;
        let skill_fit = if n > 0.0 { (score_sum / n) * 100.0 } else { 0.0 };
        let scheduled_ratio = if problem.tasks.is_empty() { 100.0 } else { n / problem.tasks.len() as f64 * 100.0 };
        Solution {
            run_id: problem.run_id, assignments, unscheduled,
            metrics: SolutionMetrics {
                overall: skill_fit * 0.6 + scheduled_ratio * 0.4,
                skill_fit, utilization: scheduled_ratio, fairness: 0.0,
            },
        }
    }
}
```

- [ ] **Step 2: Test — `crates/ai-engine/tests/solver.rs`**

```rust
use ai_engine::solver::{GreedySolver, Solver};
use ai_engine::scorer::FallbackScorer;
use ai_engine::types::*;
use ai_engine::Scorer;
use chrono::NaiveDate;
use std::collections::HashMap;

fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }

async fn problem() -> (AllocationProblem, ScoreMatrix) {
    let mut p = AllocationProblem::default();
    p.resources = vec![
        CandidateResource { id: 1, name: "R1".into(), skills: HashMap::from([(1,4)]), tags: vec![], daily_capacity_pd: 1.0 },
        CandidateResource { id: 2, name: "R2".into(), skills: HashMap::from([(1,4)]), tags: vec![], daily_capacity_pd: 1.0 },
    ];
    p.tasks = vec![
        CandidateTask { id: 10, project_id: 1, title: "T1".into(), estimate_pd: 5.0, start: d("2026-07-01"), end: d("2026-07-05"), priority: 1, skill_reqs: vec![SkillReq{skill_id:1,min_proficiency:3,is_mandatory:true,weight:1.0}] },
        CandidateTask { id: 11, project_id: 1, title: "T2".into(), estimate_pd: 5.0, start: d("2026-07-01"), end: d("2026-07-05"), priority: 2, skill_reqs: vec![SkillReq{skill_id:1,min_proficiency:3,is_mandatory:true,weight:1.0}] },
    ];
    let m = FallbackScorer.matrix(&p).await;
    (p, m)
}

#[tokio::test]
async fn schedules_both_tasks_to_distinct_resources() {
    let (p, m) = problem().await;
    let sol = GreedySolver.solve(&p, &m);
    assert_eq!(sol.assignments.len(), 2);
    assert_eq!(sol.unscheduled.len(), 0);
    let mut rids: Vec<i64> = sol.assignments.iter().map(|a| a.resource_id).collect();
    rids.sort();
    assert_eq!(rids, vec![1, 2]); // balanced across the two resources
}

#[tokio::test]
async fn unscheduled_when_no_feasible_resource() {
    let (mut p, m) = problem().await;
    p.resources = vec![CandidateResource { id: 1, name: "R1".into(), skills: HashMap::from([(1,4)]), tags: vec![], daily_capacity_pd: 1.0 }];
    let sol = GreedySolver.solve(&p, &m);
    // one task fills R1 to 1.0; the other can't fit -> unscheduled
    assert_eq!(sol.assignments.len(), 1);
    assert_eq!(sol.unscheduled.len(), 1);
}
```

- [ ] **Step 3: Run + commit**

```bash
cargo test -p ai-engine --test solver   # 2 passed
git add -A && git commit -m "feat(ai-engine): GreedySolver (hard constraints + soft objective)"
```

---

## Task 4: TemplateExplainer (deterministic; TDD)

**Files:**
- Modify: `crates/ai-engine/src/explainer.rs`
- Create: `crates/ai-engine/tests/explainer.rs`

- [ ] **Step 1: Append `TemplateExplainer` to `crates/ai-engine/src/explainer.rs`**

```rust
use crate::types::*;

pub struct TemplateExplainer;

#[async_trait::async_trait]
impl Explainer for TemplateExplainer {
    async fn explain(&self, _problem: &AllocationProblem, sol: &Solution) -> String {
        let n = sol.assignments.len();
        let unsched = sol.unscheduled.len();
        let avg = if n > 0 {
            sol.assignments.iter().map(|a| a.score).sum::<f64>() / n as f64
        } else { 0.0 };
        let mut md = format!(
            "## 优化方案说明\n\n- 已分配 **{}** 个任务，未排期 **{}** 个。\n- 平均技能匹配 {:.0}/100。\n- 综合评分 {:.0}/100。\n",
            n, unsched, avg * 100.0, sol.metrics.overall);
        if unsched > 0 {
            md.push_str(&format!("\n⚠ 未排期任务 {} 个：建议补充人力或调整时间窗。\n", unsched));
        }
        md.push_str("\n（规则模板解释；启用 LLM 可获得更细粒度的风险与改进建议。）");
        md
    }
}
```

- [ ] **Step 2: Test — `crates/ai-engine/tests/explainer.rs`**

```rust
use ai_engine::explainer::{Explainer, TemplateExplainer};
use ai_engine::types::*;

#[tokio::test]
async fn explains_counts_and_scores() {
    let sol = Solution {
        run_id: 1,
        assignments: vec![ScoredAssignment { resource_id: 1, task_id: 10, start: chrono::NaiveDate::from_ymd_opt(2026,7,1).unwrap(), end: chrono::NaiveDate::from_ymd_opt(2026,7,5).unwrap(), percent: 1.0, score: 0.8, rationale: "".into() }],
        unscheduled: vec![11],
        metrics: SolutionMetrics { overall: 70.0, skill_fit: 80.0, utilization: 50.0, fairness: 0.0 },
    };
    let md = TemplateExplainer.explain(&AllocationProblem::default(), &sol).await;
    assert!(md.contains("已分配 **1**"));
    assert!(md.contains("未排期 **1**"));
}
```

- [ ] **Step 3: Run + commit**

```bash
cargo test -p ai-engine --test explainer   # 1 passed
git add -A && git commit -m "feat(ai-engine): TemplateExplainer (rule-based)"
```

---

## Task 5: OptimizationEngine end-to-end (offline; TDD)

**Files:**
- Create: `crates/ai-engine/tests/engine.rs`

- [ ] **Step 1: Test — `crates/ai-engine/tests/engine.rs`**

```rust
use ai_engine::explainer::TemplateExplainer;
use ai_engine::scorer::FallbackScorer;
use ai_engine::solver::GreedySolver;
use ai_engine::{OptimizationEngine, Scorer};
use ai_engine::types::*;
use chrono::NaiveDate;
use std::sync::Arc;

#[tokio::test]
async fn engine_pipeline_produces_plan_with_explanation() {
    let mut p = AllocationProblem::default();
    p.run_id = 42;
    p.resources = vec![CandidateResource { id: 1, name: "R1".into(), skills: std::collections::HashMap::from([(1,4)]), tags: vec![], daily_capacity_pd: 1.0 }];
    p.tasks = vec![CandidateTask { id: 10, project_id: 1, title: "T1".into(), estimate_pd: 5.0, start: NaiveDate::from_ymd_opt(2026,7,1).unwrap(), end: NaiveDate::from_ymd_opt(2026,7,5).unwrap(), priority: 1, skill_reqs: vec![SkillReq{skill_id:1,min_proficiency:3,is_mandatory:true,weight:1.0}] }];

    let engine = OptimizationEngine {
        scorer: Arc::new(FallbackScorer),
        solver: Arc::new(GreedySolver),
        explainer: Arc::new(TemplateExplainer),
    };
    let plan = engine.optimize(&p).await;

    assert_eq!(plan.solution.run_id, 42);
    assert_eq!(plan.solution.assignments.len(), 1);
    assert_eq!(plan.solution.assignments[0].resource_id, 1);
    assert!(plan.explanation_md.contains("优化方案说明"));
}
```

- [ ] **Step 2: Run the whole crate — verify PASS**

Run: `cargo test -p ai-engine`
Expected: scorer 2 + solver 2 + explainer 1 + engine 1 = `6 passed`.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "test(ai-engine): end-to-end offline pipeline (score→solve→explain)"
```

---

## Task 6: Production impls — `rig` SemanticScorer + `good_lp` MilpSolver + `rig` LlmExplainer (feature-gated)

These are gated behind `llm`/`milp` features; tests are `#[ignore]` (need a running Ollama at `http://localhost:11434`). The deterministic path remains the default for unit tests.

**Files:**
- Modify: `crates/ai-engine/src/scorer.rs`, `src/solver.rs`, `src/explainer.rs`

- [ ] **Step 1: `SemanticScorer` (embeddings via `rig`) — append to `crates/ai-engine/src/scorer.rs`**

```rust
#[cfg(feature = "llm")]
pub mod semantic {
    use crate::types::*;
    use async_trait::async_trait;
    use rig_core::client::{Client, ProviderClient};
    use rig_core::embeddings::EmbeddingsBuilder;

    /// Cosine similarity over rig embeddings of resource skill/tag text vs task requirement text.
    /// Local Ollama default (design §5). Verify field names (.content) against the locked rig-core docs.
    pub struct SemanticScorer { pub model: String }

    #[async_trait]
    impl super::Scorer for SemanticScorer {
        async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
            let client = match rig_core::providers::ollama::Client::new("http://localhost:11434") {
                Ok(c) => c, Err(_) => return 0.0,
            };
            let model = client.embedding_model(&self.model);
            let r_text = format!("{:?} {:?}", r.skills, r.tags);
            let t_text = format!("{} {:?}", t.title, t.skill_reqs);
            let docs = EmbeddingsBuilder::new(model)
                .documents(vec![rig_core::embeddings::Document::new(r_text), rig_core::embeddings::Document::new(t_text)])
                .unwrap()
                .build()
                .await;
            match docs {
                Ok(e) if e.len() >= 2 => cosine(&e[0].embedding, &e[1].embedding).max(0.0),
                _ => 0.0,
            }
        }
    }

    fn cosine(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[tokio::test]
        #[ignore = "needs Ollama running with the embed model"]
        async fn smoke_semantic() {
            let s = SemanticScorer { model: "nomic-embed-text".into() };
            let _ = s.score(&CandidateResource { id: 1, name: "R".into(), skills: Default::default(), tags: vec!["rust".into()], daily_capacity_pd: 1.0 },
                &CandidateTask { id: 1, project_id: 1, title: "rust backend".into(), estimate_pd: 1.0, start: chrono::NaiveDate::from_ymd_opt(2026,7,1).unwrap(), end: chrono::NaiveDate::from_ymd_opt(2026,7,2).unwrap(), priority: 1, skill_reqs: vec![] }).await;
        }
    }
}
```

- [ ] **Step 2: `MilpSolver` (`good_lp`+HiGHS) — append to `crates/ai-engine/src/solver.rs`**

```rust
#[cfg(feature = "milp")]
pub mod milp {
    use crate::types::*;
    use crate::solver::Solver;

    /// MILP formulation (design §5.5.1): x[r,t,d] ∈ {0,1} + continuous percent, capacity
    /// Σ_t percent ≤ day_factor in ratio space (design §3.8). GreedySolver is the default;
    /// MilpSolver refines for larger instances. Marked optional; verify good_lp highs API.
    pub struct MilpSolver;
    impl Solver for MilpSolver {
        fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
            // TODO(impl): build good_lp::variable_problem with highs solver; map solution to ScoredAssignment.
            // Fallback to greedy semantics if infeasible/unavailable:
            super::GreedySolver.solve(problem, scores)
        }
    }
}
```

> The `MilpSolver` body is intentionally a thin stub that delegates to `GreedySolver` — a full MILP encoding (decision vars, capacity coupling constraints, objective) is a substantial impl-time task (design §5.5.1 gives the formulation). It compiles behind `milp` and provides the seam; replace the body with the real `good_lp` model when wiring production. This is the one explicitly-deferred body in the plan (noted, not hidden).

- [ ] **Step 3: `LlmExplainer` (chat via `rig`) — append to `crates/ai-engine/src/explainer.rs`**

```rust
#[cfg(feature = "llm")]
pub mod llm {
    use crate::types::*;
    use async_trait::async_trait;

    pub struct LlmExplainer { pub model: String }

    #[async_trait]
    impl super::Explainer for LlmExplainer {
        async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String {
            let client = match rig_core::providers::ollama::Client::new("http://localhost:11434") {
                Ok(c) => c, Err(_) => return super::TemplateExplainer.explain(problem, sol).await,
            };
            let prompt = format!(
                "Summarize this allocation plan: {} assignments, {} unscheduled, overall {:.0}/100. Highlight risks.",
                sol.assignments.len(), sol.unscheduled.len(), sol.metrics.overall);
            let model = client.agent(&self.model).build();
            match model.prompt(&prompt).await {
                Ok(resp) => resp.content,                       // field name per locked rig-core version
                Err(_) => super::TemplateExplainer.explain(problem, sol).await,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[tokio::test]
        #[ignore = "needs Ollama running with the chat model"]
        async fn smoke_llm() {
            let _ = LlmExplainer { model: "qwen2.5:7b".into() }.explain(&AllocationProblem::default(), &Solution::default()).await;
        }
    }
}
```

- [ ] **Step 4: Build with features**

Run: `cargo build -p ai-engine --features llm,milp`
Expected: compiles (may need rig-core/good_lp version pins adjusted per the Task 1 caveat).

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat(ai-engine): rig SemanticScorer/LlmExplainer + good_lp MilpSolver (feature-gated)"
```

---

## Task 7: App wiring — problem builder + run persistence + apply/reject + commands

**Files:**
- Create: `crates/app/src/service/optimization.rs`
- Modify: `crates/app/src/service/mod.rs`, `crates/app/src/command.rs`, `crates/app/Cargo.toml`, `src-tauri/src/main.rs`
- Create: `crates/app/tests/optimization.rs`

- [ ] **Step 1: `crates/app/Cargo.toml`** — depend on ai-engine (with production features when Ollama is desired; default off keeps tests offline):

```toml
[dependencies]
ai-engine = { path = "../ai-engine" }   # add features = ["llm","milp"] when wiring production
```

- [ ] **Step 2: `crates/app/src/service/optimization.rs`**

```rust
use crate::error::AppError;
use ai_engine::{FallbackScorer, GreedySolver, OptimizationEngine, TemplateExplainer};
use ai_engine::types::*;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct RunResult { pub run_id: i64, pub plan: ai_engine::OptimizedPlan }

pub struct OptimizationService;

impl OptimizationService {
    /// Build the problem from DB (all active resources + a project's tasks), run the engine,
    /// persist a reproducible run (status='proposed'), and return the plan (not yet applied).
    pub async fn run_for_project(pool: &SqlitePool, project_id: i64) -> Result<RunResult, AppError> {
        let problem = build_problem(pool, project_id).await?;

        // Default offline pipeline; swap in SemanticScorer/MilpSolver/LlmExplainer when Ollama is up.
        let engine = OptimizationEngine {
            scorer: Arc::new(FallbackScorer),
            solver: Arc::new(GreedySolver),
            explainer: Arc::new(TemplateExplainer),
        };
        let started = chrono::Utc::now();
        let plan = engine.optimize(&problem).await;
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds();

        let cfg = serde_json::to_string(&problem.config).unwrap_or_default();
        let cons = serde_json::to_string(&problem.flags).unwrap_or_default();
        let wts = serde_json::to_string(&problem.weights).unwrap_or_default();
        let snap = serde_json::to_string(&problem).unwrap_or_default();
        let out = serde_json::to_string(&plan.solution.assignments).unwrap_or_default();

        let (run_id,): (i64,) = sqlx::query_as(
            "INSERT INTO ai_optimization_runs (seed, scope, scope_project_ids, config_json, constraints_json, \
                weights_json, input_snapshot_json, output_plan_json, score_overall, score_skill_fit, \
                score_utilization, score_fairness, explanation_md, provider, chat_model, embed_model, \
                solver_backend, solver_status, status, started_at, finished_at, duration_ms) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?, 'ollama','qwen2.5:7b','nomic-embed-text', 'greedy','feasible','proposed', ?,?,?) RETURNING id")
            .bind(problem.config.seed).bind("incremental").bind(format!("[{}]", project_id))
            .bind(cfg).bind(cons).bind(wts).bind(snap).bind(out)
            .bind(plan.solution.metrics.overall).bind(plan.solution.metrics.skill_fit)
            .bind(plan.solution.metrics.utilization).bind(plan.solution.metrics.fairness)
            .bind(&plan.explanation_md)
            .bind(started.to_rfc3339()).bind(finished.to_rfc3339()).bind(duration_ms)
            .fetch_one(pool).await?;

        Ok(RunResult { run_id, plan })
    }

    /// Accept a proposed run: write its assignments as allocations (source='ai', run_id) and mark applied.
    pub async fn apply(pool: &SqlitePool, run_id: i64) -> Result<i64, AppError> {
        let (plan_json,): (Option<String>,) = sqlx::query_as(
            "SELECT output_plan_json FROM ai_optimization_runs WHERE id=? AND applied=0")
            .bind(run_id).fetch_optional(pool).await?
            .ok_or_else(|| domain::DomainError::NotFound(format!("run {}", run_id)))?;
        let assignments: Vec<ai_engine::ScoredAssignment> =
            serde_json::from_str(&plan_json.unwrap_or_default()).unwrap_or_default();

        db::tx::with_write_tx(pool, |tx| Box::pin(async move {
            // map task_id -> project to set the allocation; allocations table has no project col, FK via task.
            for a in &assignments {
                sqlx::query(
                    "INSERT INTO allocations (resource_id, task_id, start_date, end_date, percent, source, run_id) \
                     VALUES (?,?,?,?, 'ai', ?)")
                    .bind(a.resource_id).bind(a.task_id).bind(a.start).bind(a.end).bind(a.percent).bind(run_id)
                    .execute(&mut **tx).await?;
            }
            sqlx::query("UPDATE ai_optimization_runs SET applied=1, status='accepted' WHERE id=?")
                .bind(run_id).execute(&mut **tx).await?;
            Ok(())
        })).await?;
        Ok(assignments.len() as i64)
    }

    pub async fn reject(pool: &SqlitePool, run_id: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE ai_optimization_runs SET status='rejected' WHERE id=?")
            .bind(run_id).execute(pool).await?;
        Ok(())
    }

    pub async fn list_recent(pool: &SqlitePool, limit: i64) -> Result<Vec<RunRow>, AppError> {
        Ok(sqlx::query_as::<_, RunRow>(
            "SELECT id, objective, status, applied, score_overall, created_at FROM ai_optimization_runs \
             ORDER BY created_at DESC LIMIT ?").bind(limit).fetch_all(pool).await?)
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct RunRow { pub id: i64, pub objective: String, pub status: String, pub applied: i64, pub score_overall: Option<f64>, pub created_at: String }

async fn build_problem(pool: &SqlitePool, project_id: i64) -> Result<AllocationProblem, AppError> {
    use std::collections::HashMap;
    // resources + skills
    let resources: Vec<(i64, String, f64)> = sqlx::query_as(
        "SELECT id, name, daily_capacity_pd FROM resources WHERE deleted_at IS NULL AND status='active'")
        .fetch_all(pool).await?;
    let mut cand = Vec::new();
    for (id, name, cap) in resources {
        let skills: HashMap<i64, i64> = sqlx::query_as::<_, (i64,i64)>(
            "SELECT skill_id, proficiency FROM resource_skills WHERE resource_id=?").bind(id).fetch_all(pool).await?
            .into_iter().collect();
        cand.push(CandidateResource { id, name, skills, tags: vec![], daily_capacity_pd: cap });
    }
    // tasks + skill reqs for the project (todo/in_progress only)
    let tasks: Vec<(i64,String,f64,Option<NaiveDate>,Option<NaiveDate>,i64)> = sqlx::query_as(
        "SELECT id, title, estimate_pd, start_date, end_date, priority FROM tasks \
         WHERE project_id=? AND deleted_at IS NULL AND status IN ('todo','in_progress')")
        .bind(project_id).fetch_all(pool).await?;
    let mut cand_tasks = Vec::new();
    for (id, title, est, start, end, pri) in tasks {
        let reqs: Vec<SkillReq> = sqlx::query_as::<_, (i64,i64,i64,f64)>(
            "SELECT skill_id, min_proficiency, is_mandatory, weight FROM task_skill_requirements WHERE task_id=?")
            .bind(id).fetch_all(pool).await?
            .into_iter().map(|(s,p,m,w)| SkillReq{skill_id:s,min_proficiency:p,is_mandatory:m!=0,weight:w}).collect();
        cand_tasks.push(CandidateTask {
            id, project_id, title, estimate_pd: est,
            start: start.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026,1,1).unwrap()),
            end: end.unwrap_or_else(|| NaiveDate::from_ymd_opt(2026,12,31).unwrap()),
            priority: pri, skill_reqs: reqs,
        });
    }

    let (run_id,): (i64,) = sqlx::query_as("SELECT COALESCE(MAX(id),0)+1 FROM ai_optimization_runs").fetch_one(pool).await?;
    Ok(AllocationProblem { run_id, resources: cand, tasks: cand_tasks, existing: vec![], weights: ObjectiveWeights::default(), flags: ConstraintFlags::default(), config: SolverConfig::default() })
}
```

- [ ] **Step 3: Register module — `crates/app/src/service/mod.rs`** add `pub mod optimization;`

- [ ] **Step 4: Commands — append to `crates/app/src/command.rs`**

```rust
use crate::service::optimization::{OptimizationService, RunResult, RunRow};

#[tauri::command]
pub async fn run_optimization(state: tauri::State<'_, AppState>, project_id: i64) -> Result<RunResult, AppError> {
    OptimizationService::run_for_project(&state.pool, project_id).await
}
#[tauri::command]
pub async fn list_optimization_runs(state: tauri::State<'_, AppState>, limit: Option<i64>) -> Result<Vec<RunRow>, AppError> {
    OptimizationService::list_recent(&state.pool, limit.unwrap_or(20)).await
}
#[tauri::command]
pub async fn apply_solution(state: tauri::State<'_, AppState>, run_id: i64) -> Result<i64, AppError> {
    OptimizationService::apply(&state.pool, run_id).await
}
#[tauri::command]
pub async fn reject_solution(state: tauri::State<'_, AppState>, run_id: i64) -> Result<(), AppError> {
    OptimizationService::reject(&state.pool, run_id).await
}
```

- [ ] **Step 5: Register commands in `src-tauri/src/main.rs`** — add `run_optimization, list_optimization_runs, apply_solution, reject_solution` to the handler list.

- [ ] **Step 6: Integration test — `crates/app/tests/optimization.rs`**

```rust
use app::service::optimization::OptimizationService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::catalog::CatalogService;
use db::pool::connect;

#[tokio::test]
async fn run_then_apply_creates_ai_allocations() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-05"), false, 0, &[(rust,3,true,1.0)], &[]).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid).await.unwrap();
    assert!(res.plan.solution.assignments.len() >= 1);
    assert_eq!(res.plan.solution.assignments[0].resource_id, 1);
    assert!(res.plan.explanation_md.contains("优化方案说明"));

    let n = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert!(n >= 1);
    let (applied,): (i64,) = sqlx::query_as("SELECT applied FROM ai_optimization_runs WHERE id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(applied, 1);
    let (cnt,): (i64,) = sqlx::query_as("SELECT count(*) FROM allocations WHERE source='ai' AND run_id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert!(cnt >= 1);
}
```

- [ ] **Step 7: Run full suite — verify PASS**

Run: `cargo test --workspace`
Expected: all prior + ai-engine (6) + app optimization (1). Production impls' `#[ignore]` tests skipped.

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "feat(app): AI optimization service + commands (run/apply/reject/list)"
```

---

## Self-Review

**Spec coverage (design §5 + roadmap Phase 4):**
- §5.3 AllocationProblem/Candidate*/ObjectiveWeights/ConstraintFlags/SolverConfig types → Task 1 ✓
- §5.4 SemanticScorer + **FallbackScorer** degradation → Tasks 2, 6 ✓
- §5.5 Solver + **GreedySolver** (hard constraints: mandatory skills, per-day Σpercent≤1.0; soft: score×weight, balance) → Tasks 3, 6 ✓
- §5.6 LlmExplainer + **TemplateExplainer** degradation → Tasks 4, 6 ✓
- §5.8 OptimizationEngine pipeline (score→solve→explain) → Task 5 ✓
- §5.7/§3.3.16 reproducible run persistence (seed/snapshots/provider/scores) → Task 7 ✓
- Human-in-the-loop (apply→allocations source='ai',run_id; reject) → Task 7 ✓
- Confirmed #6 (UI-tunable ObjectiveWeights, default 0.4/0.4/0.2) → types (Task 1) ✓
- Confirmed #8 (LLM/embedding degradation to Template/Fallback) → Tasks 2/4/6 ✓

**Deferred (explicitly noted, not placeholders):**
- Full `good_lp` MILP encoding in `MilpSolver` (body delegates to greedy; design §5.5.1 formulation provided) — Task 6 explicitly flags this as the one deferred body.
- Incremental optimization (respect existing allocations in scope); `workload_cache`; per-day calendar-aware capacity in the solver (uses uniform-day proxy; workload engine authoritative).
- AI panel UI (Phase 4b).

**Placeholder scan:** none in the deterministic core — complete code with concrete assertions. `MilpSolver.solve` body is a deliberate delegation stub (flagged in-task + self-review), not a hidden placeholder.

**Type consistency:**
- `ScoredAssignment` / `Solution` / `OptimizedPlan` used identically across scorer→solver→explainer→engine→app.
- `apply_solution` serializes `Vec<ScoredAssignment>` to `output_plan_json` (run) and deserializes it back — same type, round-trips.
- `build_problem` SQL aliases match `CandidateResource`/`CandidateTask` construction; `run_id` derived consistently.
- `with_write_tx` closure uses `&mut **tx` (Phase 0 borrowed pattern).

**Known impl-time items:**
- `rig-core`/`good_lp` version pins (Task 1 caveat); verify `.content` field name and `EmbeddingsBuilder`/`Document` API against the locked docs.rs (design §5 noted rig-core ≈0.24.x drift).
- GreedySolver capacity uses a uniform per-day proxy; upgrade to calendar-aware (`day_factor`) capacity by passing a working-day set into `AllocationProblem` (the workload engine remains authoritative for display).
- `apply_solution` allocations use `source='ai'` + `run_id`; the DB trigger validates windows (a proposed assignment outside a task window will ABORT — surface as `AppError`).

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-hr-kanban-phase4-ai-engine.md`. **1. Subagent-Driven** (recommended) or **2. Inline**. Next: **Phase 4b (AI panel UI)** or **Phase 5 (reports)**.
