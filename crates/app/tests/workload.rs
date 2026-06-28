use app::service::workload::WorkloadService;
use db::{AllocationsRepo, ResourcesRepo};

async fn fresh() -> sqlx::SqlitePool {
    let pool = db::pool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

const MON: &str = "2026-06-29"; const FRI: &str = "2026-07-03";

#[tokio::test]
async fn resource_summary_half_loaded_with_holiday() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // global full-day holiday Wed
    sqlx::query("INSERT INTO holiday (project_id,day,fraction,name) VALUES (NULL,'2026-07-01',1.0,'H')").execute(&pool).await.unwrap();
    // Alice 50% Mon..Fri
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 0.5).await.unwrap();

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI).await.unwrap();
    // capacity = 4.0 (Wed=0); workload = 4*0.5 = 2.0; utilization = 0.5
    assert!((s.capacity_pd - 4.0).abs() < 1e-9);
    assert!((s.workload_pd - 2.0).abs() < 1e-9);
    assert!((s.utilization - 0.5).abs() < 1e-9);
    assert!(!s.overloaded); // 0.5 < 1.10
}

#[tokio::test]
async fn detects_overload_with_two_full_allocations() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'Q')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,2,'U','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap(); // 100% on P
    AllocationsRepo::create(&pool, 1, 11, MON, FRI, 1.0).await.unwrap(); // 100% on Q -> 200%

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI).await.unwrap();
    assert!(s.utilization > 1.0);
    assert!(s.overloaded);

    let ov = WorkloadService::overloads(&pool, MON, FRI).await.unwrap();
    assert_eq!(ov.len(), 1);
    assert_eq!(ov[0].resource_id, 1);
}

#[tokio::test]
async fn bad_window_rejected() {
    let pool = fresh().await;
    // need a resource id=1 to exist so the lookup doesn't short-circuit on NotFound;
    // but resource_summary parses the window before any DB read, so no row needed.
    let _ = ResourcesRepo::create(&pool, "X", None).await.unwrap();
    let err = WorkloadService::resource_summary(&pool, 1, "2026-07-05", "2026-06-01").await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
