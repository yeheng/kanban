use app::service::optimization::OptimizationService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::catalog::CatalogService;
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
    let (applied,): (i64,) = sqlx::query_as("SELECT applied FROM ai_optimization_runs WHERE id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(applied, 1);
    let (cnt,): (i64,) = sqlx::query_as("SELECT count(*) FROM allocations WHERE source='ai' AND run_id=?").bind(res.run_id).fetch_one(&pool).await.unwrap();
    assert!(cnt >= 1);
}

/// Changing objective weights produces a different outcome: with budget weight set high
/// enough to dominate, the greedy budget-cap gate activates (planned PD capped at
/// budget_pd), scheduling fewer tasks than the balanced default. Pins G3's
/// "weight-sensitive optimization" (design §5).
#[tokio::test]
async fn budget_weight_dominating_caps_planned_pd() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    // Small budget so the cap is binding when budget weight dominates.
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 5.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    // Two resources (so balanced can schedule BOTH tasks — one per resource).
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (2,'Bob')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (2,?,4)").bind(rust).execute(&pool).await.unwrap();
    // Two 5-PD tasks, both fit the skills & window.
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();
    TasksService::create(&pool, pid, "T2", None, 5.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    // Balanced weights (budget 0.2): no budget cap → both tasks scheduled (planned 10 PD).
    let balanced = OptimizationService::run_for_project(&pool, pid, Some(ObjectiveWeights { skill_fit: 0.4, balance: 0.4, budget: 0.2 })).await.unwrap();
    let n_balanced = balanced.plan.solution.assignments.len();

    // Budget-dominant weights (budget 0.9 > max(0.05,0.05)): budget cap activates → only
    // tasks within the 5-PD budget are scheduled (one task), the other goes unscheduled.
    let budgety = OptimizationService::run_for_project(&pool, pid, Some(ObjectiveWeights { skill_fit: 0.05, balance: 0.05, budget: 0.9 })).await.unwrap();
    let n_budget = budgety.plan.solution.assignments.len();

    assert!(n_balanced > n_budget, "budget-dominant weights schedule fewer tasks: balanced={} budget={}", n_balanced, n_budget);
    assert_eq!(n_budget, 1, "5-PD budget caps at one 5-PD task");
    assert_eq!(n_balanced, 2, "balanced schedules both tasks (one per resource)");
}
