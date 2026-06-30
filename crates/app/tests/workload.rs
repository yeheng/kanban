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

// ---- P1 #2: project calendar overrides affect utilization (design §4.9) ----

/// A project-scoped holiday reduces that project's allocation PD (numerator) while the
/// global capacity (denominator) is unchanged, so utilization drops. This pins the
/// "project-level override affects utilization" property and the global-denominator policy.
#[tokio::test]
async fn project_holiday_changes_utilization_not_global_capacity() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // Wed is a holiday ONLY for project 1 (project-scoped).
    sqlx::query("INSERT INTO holiday (project_id,day,fraction,name) VALUES (1,'2026-07-01',1.0,'ProjHoliday')")
        .execute(&pool).await.unwrap();
    // Alice 100% Mon..Fri on project 1.
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap();

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI).await.unwrap();
    // Denominator: global capacity = 5.0 (Mon-Fri, no global holiday).
    assert!((s.capacity_pd - 5.0).abs() < 1e-9, "global capacity unaffected: {}", s.capacity_pd);
    // Numerator: project-1 allocation PD = 4.0 (Wed dropped by project holiday).
    assert!((s.workload_pd - 4.0).abs() < 1e-9, "workload reflects project holiday: {}", s.workload_pd);
    // Utilization = 4.0 / 5.0 = 0.8.
    assert!((s.utilization - 0.8).abs() < 1e-9, "utilization = {}", s.utilization);
}

/// Capacity-conflict detection must rate each existing allocation against its OWN
/// project's calendar (bug fix). Distinctive case: the existing allocation's project
/// (P2) has a Wed holiday, the new allocation's project (P1) does not. Both allocations
/// are 100% Mon..Fri. Pre-fix, the existing P2 load on Wed was rated against P1's
/// calendar (day_factor>0) and counted on top of the new 100% — but that alone wouldn't
/// flip the verdict since Mon/Tue/Thu/Fri already conflict. So instead we scope the new
/// allocation to Wed ONLY: pre-fix the P2 Wed load (wrongly counted under P1) + new 100%
/// > 1.0 ⇒ wrongly rejected; post-fix the P2 Wed load is dropped (P2 holiday) and only
/// the new 100% counts ⇒ accepted. This is the case that distinguishes the fix.
#[tokio::test]
async fn capacity_accepts_when_existing_alloc_holiday_drops_load_under_own_calendar() {
    use app::service::allocations::AllocationsService;
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P1')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'P2')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    // P2 has a Wed holiday; P1 does not.
    sqlx::query("INSERT INTO holiday (project_id,day,fraction,name) VALUES (2,'2026-07-01',1.0,'P2Wed')")
        .execute(&pool).await.unwrap();
    // Existing allocation: Alice 100% on P2, Mon..Fri.
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,2,'T2','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap();

    // New allocation on P1 (no Wed holiday) at 100%, Wed ONLY.
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,1,'T1','2026-07-01','2026-07-01')").execute(&pool).await.unwrap();
    // Pre-fix: P2's Wed load counted under P1 → 100% (existing) + 100% (new) = 200% > 1.0 ⇒ rejected (WRONG).
    // Post-fix: P2's Wed load dropped (P2 holiday) → only 100% (new) ≤ 1.0 ⇒ accepted (CORRECT).
    AllocationsService::create(&pool, 1, 11, "2026-07-01", "2026-07-01", 1.0)
        .await
        .expect("accepted: existing P2 Wed load dropped by its own-project holiday");
}
