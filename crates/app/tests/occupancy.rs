use app::service::occupancy::CalendarOccupancyService;
use chrono::Datelike;
use db::pool::connect;
use db::AllocationsRepo;

#[tokio::test]
async fn daily_occupancy_for_half_loaded_week() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // Alice 50% Mon..Fri (2026-06-29..2026-07-03)
    AllocationsRepo::create(&pool, 1, 10, "2026-06-29", "2026-07-03", 0.5).await.unwrap();

    let occ = CalendarOccupancyService::range(&pool, "2026-06-29", "2026-07-05").await.unwrap();
    // 5 working days (Mon-Fri); each: workload 0.5, capacity 1.0, utilization 0.5
    assert_eq!(occ.len(), 5);
    for o in &occ {
        assert!((o.workload_pd - 0.5).abs() < 1e-9);
        assert!((o.capacity_pd - 1.0).abs() < 1e-9);
        assert!((o.utilization - 0.5).abs() < 1e-9);
    }
    // weekend days (cap 0) are skipped
    assert!(occ.iter().all(|o| o.date.weekday().num_days_from_monday() < 5));
}
