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
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-01"), Some("2026-07-05"), false, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    let res = OptimizationService::run_for_project(&pool, pid).await.unwrap();
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
