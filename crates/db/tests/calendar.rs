use db::pool::connect;
use db::repo::calendar::{hydrate, HolidayRepo, TimeOffRepo, WeekTemplateRepo};
use domain::{capacity_pd, Window};
use chrono::NaiveDate;

fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
fn win(s: &str, e: &str) -> Window { Window { start: d(s), end: d(e) } }

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn hydrate_reflects_db_calendar() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();

    // global week Mon-Fri (already seeded by 0001, but re-assert via upsert)
    WeekTemplateRepo::upsert_global(&pool, [1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]).await.unwrap();
    // global full-day holiday on Wed 2026-07-01
    HolidayRepo::add(&pool, None, "2026-07-01", 1.0, Some("Holiday")).await.unwrap();
    // Alice half-day time-off Thu 2026-07-02
    TimeOffRepo::add(&pool, 1, "2026-07-02", 0.5, Some("leave")).await.unwrap();

    let cal = hydrate(&pool).await.unwrap();
    // capacity Mon..Fri with Wed holiday + Thu half: 1+1+0+0.5+1 = 3.5
    let cap = capacity_pd(&cal, 1, 1, win("2026-06-29", "2026-07-03"));
    assert!((cap - 3.5).abs() < 1e-9);
}

#[tokio::test]
async fn hydrate_empty_returns_default_calendar() {
    let pool = fresh().await;
    let cal = hydrate(&pool).await.unwrap();
    // seed gives Mon-Fri -> capacity 5.0 PD for a Mon–Fri week
    let cap = capacity_pd(&cal, 1, 1, win("2026-06-29", "2026-07-03"));
    assert!((cap - 5.0).abs() < 1e-9);
}
