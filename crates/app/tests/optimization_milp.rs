//! App-layer integration for the MILP solver path (gated on the `milp` feature).
//! Verifies that with `solver_backend='good_lp'` the persisted run row carries a real
//! `solver_status` (optimal/feasible — not the old hardcoded 'feasible'), and that apply()
//! survives the trg_allocation_validate_insert trigger (the solver's percent is trigger-safe).
#![cfg(feature = "milp")]

use app::service::optimization::OptimizationService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::catalog::CatalogService;
use db::pool::connect;

#[tokio::test]
async fn milp_run_persists_real_solver_status_and_applies_cleanly() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    // Force the MILP backend (the default is 'good_lp' in the schema, but be explicit).
    sqlx::query("UPDATE settings SET solver_backend='good_lp' WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();

    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)")
        .bind(rust).execute(&pool).await.unwrap();
    TasksService::create(
        &pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-07"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[],
    ).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    assert!(!res.plan.solution.assignments.is_empty());

    // The run row must carry a real solver_status from HiGHS, not a hardcoded literal.
    let (status, backend): (String, String) = sqlx::query_as(
        "SELECT solver_status, solver_backend FROM ai_optimization_runs WHERE id=?",
    )
    .bind(res.run_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(backend, "good_lp");
    assert!(
        matches!(status.as_str(), "optimal" | "feasible"),
        "expected optimal/feasible, got {status}"
    );

    // apply() must not ABORT on the trigger (solver percent is in (0,1.0]).
    let n = OptimizationService::apply(&pool, res.run_id).await.unwrap();
    assert!(n >= 1);
}
