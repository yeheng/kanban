use app::service::workload::WorkloadService;
use db::{AllocationsRepo, ResourcesRepo};

async fn fresh() -> sqlx::SqlitePool {
    let pool = db::pool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

const MON: &str = "2026-06-29";
const FRI: &str = "2026-07-03";

#[tokio::test]
async fn resource_summary_half_loaded_with_holiday() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // global full-day holiday Wed
    sqlx::query(
        "INSERT INTO holiday (project_id,day,fraction,name) VALUES (NULL,'2026-07-01',1.0,'H')",
    )
    .execute(&pool)
    .await
    .unwrap();
    // Alice 50% Mon..Fri
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 0.5)
        .await
        .unwrap();

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI)
        .await
        .unwrap();
    // capacity = 4.0 (Wed=0); workload = 4*0.5 = 2.0; utilization = 0.5
    assert!((s.capacity_pd - 4.0).abs() < 1e-9);
    assert!((s.workload_pd - 2.0).abs() < 1e-9);
    assert!((s.utilization - 0.5).abs() < 1e-9);
    assert!(!s.overloaded); // 0.5 < 1.10
}

#[tokio::test]
async fn detects_overload_with_two_full_allocations() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'Q')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,2,'U','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0)
        .await
        .unwrap(); // 100% on P
    AllocationsRepo::create(&pool, 1, 11, MON, FRI, 1.0)
        .await
        .unwrap(); // 100% on Q -> 200%

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI)
        .await
        .unwrap();
    assert!(s.utilization > 1.0);
    assert!(s.overloaded);

    let ov = WorkloadService::overloads(&pool, MON, FRI).await.unwrap();
    assert_eq!(ov.len(), 1);
    assert_eq!(ov[0].resource_id, 1);
}

#[tokio::test]
async fn bad_window_rejected() {
    let pool = fresh().await;
    // The resource must exist so this exercises window validation rather than NotFound.
    let _ = ResourcesRepo::create(&pool, "X", None).await.unwrap();
    let err = WorkloadService::resource_summary(&pool, 1, "2026-07-05", "2026-06-01")
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn resource_summary_rejects_missing_resource() {
    let pool = fresh().await;
    let err = WorkloadService::resource_summary(&pool, 999, MON, FRI)
        .await
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

use app::service::projects::ProjectsService;
use app::service::teams::TeamsService;

#[tokio::test]
async fn team_summary_aggregates_members() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (2,'B')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // A overloaded (100% on two tasks), B idle
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,1,'U','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0)
        .await
        .unwrap();
    AllocationsRepo::create(&pool, 1, 11, MON, FRI, 1.0)
        .await
        .unwrap();

    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, 1, None).await.unwrap();
    TeamsService::add_member(&pool, tid, 2, None).await.unwrap();

    let s = WorkloadService::team_summary(&pool, tid, MON, FRI)
        .await
        .unwrap();
    // capacity = 5 (A) + 5 (B) = 10; workload = 10 (A) + 0 (B) = 10; util = 1.0
    assert!((s.capacity_pd - 10.0).abs() < 1e-9);
    assert!((s.workload_pd - 10.0).abs() < 1e-9);
    assert!((s.utilization - 1.0).abs() < 1e-9);
    assert_eq!(s.overloaded_members, vec![1]); // A util=2.0 > 1.10; B=0
}

#[tokio::test]
async fn team_summary_rejects_missing_team() {
    let pool = fresh().await;
    let err = WorkloadService::team_summary(&pool, 999, MON, FRI)
        .await
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[tokio::test]
async fn project_burn_ratio() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 40.0)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,?,'T','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    // 100% for Mon..Fri -> allocated_pd = 5 * 1.0 = 5.0 (full-span, global Mon-Fri calendar)
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0)
        .await
        .unwrap();
    let b = WorkloadService::project_burn(&pool, pid).await.unwrap();
    assert!((b.allocated_pd - 5.0).abs() < 1e-9);
    assert!((b.usage - (5.0 / 40.0)).abs() < 1e-9);
}
