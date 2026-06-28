use db::pool::connect;
use db::repo::AllocationsRepo;
use domain::{workload_pd, Calendar, DayFraction, Window};
use chrono::NaiveDate;

fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }

#[tokio::test]
async fn persist_then_compute_workload() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // project 1 (Jun), resource 1, task 10 in project 1 (Jun 8-19)
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-06-30')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-08','2026-06-19')").execute(&pool).await.unwrap();

    // 50% allocation Mon 2026-06-08 .. Fri 2026-06-12 (5 workdays)
    let aid = AllocationsRepo::create(&pool, 1, 10, "2026-06-08", "2026-06-12", 0.5).await.unwrap();
    assert!(aid > 0);

    let rows = AllocationsRepo::list_for_resource(&pool, 1, "2026-06-08", "2026-06-12").await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].project_id, 1);

    // bridge to domain and compute workload for that week
    let cal = Calendar::global(DayFraction::MON_FRI);
    let allocs = vec![rows[0].to_domain()];
    let wl = workload_pd(&cal, &allocs, 1, Window { start: d("2026-06-08"), end: d("2026-06-12") });
    assert!((wl - 2.5).abs() < 1e-9); // 5 * 0.5 * 1.0
}

#[tokio::test]
async fn out_of_window_insert_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-10','2026-06-20')").execute(&pool).await.unwrap();
    let res = AllocationsRepo::create(&pool, 1, 10, "2026-06-01", "2026-06-15", 0.5).await;
    assert!(res.is_err());
}