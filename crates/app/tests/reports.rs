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

use app::service::reports::ReportTable;

#[test]
fn csv_has_header_and_rows() {
    let t = ReportTable { title: "X".into(), columns: vec!["a".into(), "b".into()], rows: vec![vec!["1".into(), "2".into()]] };
    let bytes = ReportService::to_csv(&t).unwrap();
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("a,b"));
    assert!(s.contains("1,2"));
}

#[test]
fn xlsx_is_zip() {
    let t = ReportTable { title: "X".into(), columns: vec!["a".into()], rows: vec![vec!["1".into()]] };
    let bytes = ReportService::to_xlsx(&t).unwrap();
    assert_eq!(&bytes[..2], b"PK"); // xlsx is a ZIP
}

#[cfg(feature = "pdf")]
#[test]
fn pdf_starts_with_header() {
    let t = ReportTable { title: "X".into(), columns: vec!["a".into()], rows: vec![vec!["1".into()]] };
    let bytes = ReportService::to_pdf(&t).unwrap();
    assert_eq!(&bytes[..4], b"%PDF"); // PDF magic
}

// ---- P2 #2: team report, catalog, snapshot shape ----

use app::service::teams::TeamsService;

#[tokio::test]
async fn team_utilization_report_aggregates() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'A')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (2,'B')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,?,'T','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,?,'U','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    // A overloaded (100% on two tasks), B idle.
    AllocationsRepo::create(&pool, 1, 10, "2026-06-29", "2026-07-03", 1.0).await.unwrap();
    AllocationsRepo::create(&pool, 1, 11, "2026-06-29", "2026-07-03", 1.0).await.unwrap();
    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, 1, None).await.unwrap();
    TeamsService::add_member(&pool, tid, 2, None).await.unwrap();

    let t = ReportService::build(&pool, ReportKind::TeamUtilization, None, "2026-06-29", "2026-07-03").await.unwrap();
    assert_eq!(t.rows.len(), 1, "one team");
    // columns: team, overloaded_members, capacity_pd, workload_pd, utilization, capacity_pm, workload_pm
    assert_eq!(t.rows[0][0], "Eng");
    assert_eq!(t.rows[0][1], "1", "one overloaded member (A)");
    assert!(t.columns.contains(&"capacity_pm".to_string()), "PM columns present");
}

#[test]
fn report_catalog_lists_all_kinds_with_formats() {
    let cat = ReportService::report_catalog();
    let kinds: Vec<&str> = cat.iter().map(|e| e.kind.as_str()).collect();
    assert!(kinds.contains(&"ResourceUtilization"));
    assert!(kinds.contains(&"TeamUtilization"));
    assert!(kinds.contains(&"ProjectBurn"));
    assert!(kinds.contains(&"Cost"));
    assert!(kinds.contains(&"AiDecisions"));
    // csv + xlsx always available; pdf only behind feature.
    for e in &cat {
        assert!(e.formats.contains(&"csv".to_string()), "{} has csv", e.kind);
        assert!(e.formats.contains(&"xlsx".to_string()), "{} has xlsx", e.kind);
        assert!(e.mvp, "{} marked MVP", e.kind);
    }
    // Cost is the only kind accepting a project_id filter.
    let cost = cat.iter().find(|e| e.kind == "Cost").unwrap();
    assert!(cost.accepts_project_id);
}

#[tokio::test]
async fn snapshot_json_has_documented_shape() {
    // Snapshot = point-in-time per-resource utilization over a window (design §8).
    // Shape: { window: {start,end}, resources: [{resource, capacity_pd, workload_pd,
    // utilization, overloaded}] }.
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    let s = ReportService::snapshot_json(&pool, "2026-06-29", "2026-07-03").await.unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["window"]["start"], "2026-06-29");
    assert_eq!(v["window"]["end"], "2026-07-03");
    let res = v["resources"].as_array().unwrap();
    assert_eq!(res.len(), 1);
    let r = &res[0];
    assert_eq!(r["resource"], "Alice");
    assert!(r["capacity_pd"].is_number());
    assert!(r["workload_pd"].is_number());
    assert!(r["utilization"].is_number());
    assert!(r["overloaded"].is_boolean());
}
