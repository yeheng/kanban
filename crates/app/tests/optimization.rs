use app::service::optimization::OptimizationService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::catalog::CatalogService;
use app::service::calendar::CalendarService;
use app::service::resources::ResourcesService;
use ai_engine::types::ObjectiveWeights;
use db::pool::connect;

#[tokio::test]
async fn run_then_apply_creates_ai_allocations() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    assert!(!res.plan.solution.assignments.is_empty());
    assert_eq!(res.plan.solution.assignments[0].resource_id, 1);
    assert!(res.plan.explanation_md.contains("优化方案说明"));

    let n = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert!(n >= 1);
    let n2 = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert_eq!(n2, 0, "apply is idempotent after a run is accepted");
    let (applied,): (i64,) = sqlx::query_as("SELECT applied FROM ai_optimization_runs WHERE id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(applied, 1);
    let (cnt,): (i64,) = sqlx::query_as("SELECT count(*) FROM allocations WHERE source='ai' AND run_id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(cnt, n);
}

/// Budget is an unconditional HARD gate: when `budget_pd > 0`, planned PD is capped
/// at the budget regardless of objective weights. Previously the gate fired only when
/// `weights.budget` dominated the others — a weight-gated "constraint" that made budget
/// decorative under the default balanced weights (0.2). With the same 5-PD budget and
/// two 5-PD tasks, BOTH the balanced and the budget-dominant run cap at one task.
#[tokio::test]
async fn budget_is_a_hard_gate_regardless_of_weights() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    // Small budget so the cap is binding: 5-PD budget vs two 5-PD tasks.
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 5.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    // Two resources (so capacity is not the limiter — the budget is).
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (2,'Bob')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (2,?,4)").bind(rust).execute(&pool).await.unwrap();
    // Two 5-PD tasks, both fit the skills & window.
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();
    TasksService::create(&pool, pid, "T2", None, 5.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    // Balanced weights (budget 0.2): the hard gate still fires — only one task fits the 5-PD budget.
    let balanced = OptimizationService::run_for_project(&pool, pid, Some(ObjectiveWeights { skill_fit: 0.4, balance: 0.4, budget: 0.2 })).await.unwrap();
    let n_balanced = balanced.plan.solution.assignments.len();

    // Budget-dominant weights: same hard gate, same cap.
    let budgety = OptimizationService::run_for_project(&pool, pid, Some(ObjectiveWeights { skill_fit: 0.05, balance: 0.05, budget: 0.9 })).await.unwrap();
    let n_budget = budgety.plan.solution.assignments.len();

    assert_eq!(n_balanced, 1, "balanced weights: budget hard gate caps at one 5-PD task");
    assert_eq!(n_budget, 1, "budget-dominant weights: same hard gate, same cap");
}

#[tokio::test]
async fn resource_tags_loaded_from_db_drive_assignment_choice() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_tag(&pool, "rust", None).await.unwrap();
    let alice = ResourcesService::create(&pool, "Alice", None).await.unwrap();
    let bob = ResourcesService::create(&pool, "Bob", None).await.unwrap();
    ResourcesService::set_tags(&pool, alice, &[rust]).await.unwrap();
    TasksService::create(
        &pool, pid, "rust backend", None, 1.0, Some("2026-07-01"), Some("2026-07-02"),
        false, None, None, 0, &[], &[],
    ).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();

    assert_eq!(res.plan.solution.assignments.len(), 1);
    assert_eq!(res.plan.solution.assignments[0].resource_id, alice);
    assert_ne!(res.plan.solution.assignments[0].resource_id, bob);
    assert!(res.plan.solution.assignments[0].score > 0.0);
}

#[tokio::test]
async fn unavailable_high_score_resource_is_not_assigned() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_tag(&pool, "rust", None).await.unwrap();
    let unavailable = ResourcesService::create(&pool, "Unavailable", None).await.unwrap();
    let available = ResourcesService::create(&pool, "Available", None).await.unwrap();
    ResourcesService::set_tags(&pool, unavailable, &[rust]).await.unwrap();
    ResourcesService::update(
        &pool, unavailable, "Unavailable", None,
        Some("2026-08-01"), Some("2026-08-31"), None, None,
    ).await.unwrap();
    ResourcesService::update(
        &pool, available, "Available", None,
        Some("2026-07-01"), Some("2026-07-31"), None, None,
    ).await.unwrap();
    TasksService::create(
        &pool, pid, "rust backend", None, 1.0, Some("2026-07-01"), Some("2026-07-02"),
        false, None, None, 0, &[], &[],
    ).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();

    assert_eq!(res.plan.solution.assignments.len(), 1);
    assert_eq!(res.plan.solution.assignments[0].resource_id, available);
}

#[tokio::test]
async fn calendar_capacity_changes_ai_percent_and_apply_persists() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let alice = ResourcesService::create(&pool, "Alice", None).await.unwrap();
    ResourcesService::set_skills(&pool, alice, &[(rust, 4)]).await.unwrap();
    CalendarService::add_holiday(&pool, Some(pid), "2026-07-08", Some(1.0), Some("Project holiday")).await.unwrap();
    TasksService::create(
        &pool, pid, "T1", None, 3.0, Some("2026-07-06"), Some("2026-07-10"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[],
    ).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();

    assert_eq!(res.plan.solution.assignments.len(), 1);
    let a = &res.plan.solution.assignments[0];
    assert!((a.percent - 0.75).abs() < 1e-9, "3 PD over 4 working days should require 75%, got {}", a.percent);
    assert_eq!(OptimizationService::apply(&pool, res.run_id).await.unwrap(), 1);
    let (percent,): (f64,) = sqlx::query_as("SELECT percent FROM allocations WHERE run_id=?")
        .bind(res.run_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((percent - 0.75).abs() < 1e-9);
}

#[tokio::test]
async fn concurrent_apply_writes_allocations_once() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let alice = ResourcesService::create(&pool, "Alice", None).await.unwrap();
    ResourcesService::set_skills(&pool, alice, &[(rust, 4)]).await.unwrap();
    TasksService::create(
        &pool, pid, "T1", None, 1.0, Some("2026-07-01"), Some("2026-07-02"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[],
    ).await.unwrap();
    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();

    let (first, second) = tokio::join!(
        OptimizationService::apply(&pool, res.run_id),
        OptimizationService::apply(&pool, res.run_id),
    );
    let total = first.unwrap() + second.unwrap();
    let (cnt,): (i64,) = sqlx::query_as("SELECT count(*) FROM allocations WHERE source='ai' AND run_id=?")
        .bind(res.run_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(total, 1);
    assert_eq!(cnt, 1);
}

/// TOCTOU: `run` snapshots task windows; `apply` inserts later. If a task's window is
/// narrowed between the two, the trigger would ABORT the whole batch (losing every AI
/// allocation, not just the now-out-of-window one). apply() must re-read the current windows
/// inside its write tx and SKIP out-of-window assignments instead of aborting.
#[tokio::test]
async fn apply_skips_out_of_window_after_task_window_narrowed() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 100.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)")
        .bind(rust).execute(&pool).await.unwrap();
    // Two tasks, both 1 PD, distinct days so both schedule to Alice.
    let t1 = TasksService::create(
        &pool, pid, "T1", None, 1.0, Some("2026-07-01"), Some("2026-07-01"),
        false, None, None, 0, &[], &[],
    ).await.unwrap();
    let _t2 = TasksService::create(
        &pool, pid, "T2", None, 1.0, Some("2026-07-02"), Some("2026-07-02"),
        false, None, None, 0, &[], &[],
    ).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    assert_eq!(res.plan.solution.assignments.len(), 2);

    // Narrow T1's window to 2026-07-05 so its allocation (2026-07-01) is now out of range
    // (still satisfies the tasks CHECK: end_date >= start_date).
    sqlx::query("UPDATE tasks SET start_date='2026-07-05', end_date='2026-07-05' WHERE id=?")
        .bind(t1).execute(&pool).await.unwrap();

    // apply() must NOT abort — it skips the out-of-window T1 and writes T2.
    let n = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert_eq!(n, 1, "the in-window T2 should still be written; T1 skipped");
    let (cnt,): (i64,) = sqlx::query_as("SELECT count(*) FROM allocations WHERE source='ai' AND run_id=?")
        .bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(cnt, 1);
    // The run is still marked applied (idempotent no-op on re-apply).
    let (applied,): (i64,) = sqlx::query_as("SELECT applied FROM ai_optimization_runs WHERE id=?")
        .bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(applied, 1);
    let again = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert_eq!(again, 0, "re-apply is idempotent");
}

/// When the DB requests `good_lp` but the `milp` feature is NOT compiled in, the run must
/// still succeed (greedy fallback) AND persist the backend that ACTUALLY ran (`greedy`), not
/// the requested `good_lp` — so run rows never mislabel the backend. (Guards task F: the
/// `effective_backend` returned by `select_solver` is what's persisted.)
#[cfg(not(feature = "milp"))]
#[tokio::test]
async fn run_persists_effective_backend_when_feature_missing() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    // The schema default is solver_backend='good_lp', but this binary has no milp feature.
    let (db_backend,): (String,) = sqlx::query_as("SELECT solver_backend FROM settings WHERE id=1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(db_backend, "good_lp", "schema default is good_lp");
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)")
        .bind(rust).execute(&pool).await.unwrap();
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-07"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    let (persisted,): (String,) = sqlx::query_as("SELECT solver_backend FROM ai_optimization_runs WHERE id=?")
        .bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(persisted, "greedy",
        "without the milp feature the run must record 'greedy' (the backend that actually ran), not the requested 'good_lp'");
    assert!(!res.plan.solution.assignments.is_empty());
}

/// `constraints_json` is backfilled with the dependency edges the run honored (task F: the
/// column was previously bound '' and left vestigial). With a dependency edge present, it must
/// serialize that edge so the run is reproducible.
#[tokio::test]
async fn run_persists_dependency_edges_in_constraints_json() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 100.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)")
        .bind(rust).execute(&pool).await.unwrap();
    let t1 = TasksService::create(&pool, pid, "T1", None, 1.0, Some("2026-07-01"), Some("2026-07-01"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();
    let t2 = TasksService::create(&pool, pid, "T2", None, 1.0, Some("2026-07-02"), Some("2026-07-02"),
        false, None, None, 1, &[(rust, 3, true, 1.0)], &[]).await.unwrap();
    TasksService::add_dependency(&pool, t2, t1, 0, "FS").await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    let (cons,): (Option<String>,) = sqlx::query_as("SELECT constraints_json FROM ai_optimization_runs WHERE id=?")
        .bind(res.run_id).fetch_one(&pool).await.unwrap();
    let cons = cons.unwrap_or_default();
    assert!(cons.contains("\"task_id\""), "constraints_json should serialize dependency edges, got: {cons}");
    assert!(cons.contains("\"predecessor_id\""), "constraints_json missing predecessor_id, got: {cons}");
}
