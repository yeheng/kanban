use app::service::reports::{ReportKind, ReportService};
use app::service::projects::ProjectsService;
use db::pool::connect;
use db::AllocationsRepo;

#[tokio::test]
async fn resource_utilization_and_cost_reports() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 40.0).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name,daily_rate_pd) VALUES (1,'Alice',100.0)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,?,'T','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, "2026-06-29", "2026-07-03", 1.0).await.unwrap(); // 5 PD (Mon-Fri)

    let ru = ReportService::build(&pool, ReportKind::ResourceUtilization, None, "2026-06-29", "2026-07-03").await.unwrap();
    assert!(ru.columns.contains(&"utilization".to_string()));
    assert_eq!(ru.rows.len(), 1);

    let cost = ReportService::build(&pool, ReportKind::Cost, Some(pid), "2026-06-29", "2026-07-03").await.unwrap();
    // last row is TOTAL; 5 PD * 100 = 500
    let total = cost.rows.last().unwrap().last().unwrap();
    assert!(total.contains("500"), "expected 500 in {total}");
}

#[tokio::test]
async fn snapshot_json_is_valid() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let s = ReportService::snapshot_json(&pool, "2026-06-29", "2026-07-03").await.unwrap();
    assert!(s.contains("\"resources\""));
    serde_json::from_str::<serde_json::Value>(&s).unwrap();
}

#[tokio::test]
async fn ai_decisions_empty_when_no_runs() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let t = ReportService::build(&pool, ReportKind::AiDecisions, None, "2026-06-29", "2026-07-03").await.unwrap();
    assert!(t.rows.is_empty());
    assert_eq!(t.columns.len(), 5);
}
