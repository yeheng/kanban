use app::service::catalog::CatalogService;
use app::service::resources::ResourcesService;
use db::pool::connect;

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn set_and_list_skills_round_trip() {
    let pool = fresh().await;
    let id = ResourcesService::create(&pool, "Alice", None).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let fe = CatalogService::ensure_skill(&pool, "Frontend").await.unwrap();

    // Set two skills with proficiency.
    ResourcesService::set_skills(&pool, id, &[(rust, 4), (fe, 2)]).await.unwrap();
    let skills = ResourcesService::list_skills(&pool, id).await.unwrap();
    assert_eq!(skills.len(), 2);
    // Names resolved + proficiency persisted.
    let rust_row = skills.iter().find(|s| s.skill_name == "Rust").unwrap();
    assert_eq!(rust_row.proficiency, 4);
    let fe_row = skills.iter().find(|s| s.skill_name == "Frontend").unwrap();
    assert_eq!(fe_row.proficiency, 2);

    // Replace-all: drop Frontend, keep Rust at new proficiency.
    ResourcesService::set_skills(&pool, id, &[(rust, 5)]).await.unwrap();
    let skills = ResourcesService::list_skills(&pool, id).await.unwrap();
    assert_eq!(skills.len(), 1, "set_skills replaces, not appends");
    assert_eq!(skills[0].skill_name, "Rust");
    assert_eq!(skills[0].proficiency, 5);
}

#[tokio::test]
async fn set_skills_rejects_proficiency_out_of_range() {
    let pool = fresh().await;
    let id = ResourcesService::create(&pool, "Bob", None).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let err = ResourcesService::set_skills(&pool, id, &[(rust, 6)]).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
    let err = ResourcesService::set_skills(&pool, id, &[(rust, 0)]).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn set_skills_rejects_unknown_skill() {
    let pool = fresh().await;
    let id = ResourcesService::create(&pool, "Bob", None).await.unwrap();
    let err = ResourcesService::set_skills(&pool, id, &[(999, 3)]).await.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[tokio::test]
async fn set_and_list_tags_round_trip() {
    let pool = fresh().await;
    let id = ResourcesService::create(&pool, "Alice", None).await.unwrap();
    let t1 = CatalogService::ensure_tag(&pool, "high-perf", null()).await.unwrap();
    let t2 = CatalogService::ensure_tag(&pool, "web", null()).await.unwrap();

    ResourcesService::set_tags(&pool, id, &[t1, t2]).await.unwrap();
    let tags = ResourcesService::list_tags(&pool, id).await.unwrap();
    assert_eq!(tags.len(), 2);
    let names: Vec<&str> = tags.iter().map(|t| t.tag_name.as_str()).collect();
    assert!(names.contains(&"high-perf"));
    assert!(names.contains(&"web"));

    // Replace-all: keep only web.
    ResourcesService::set_tags(&pool, id, &[t2]).await.unwrap();
    let tags = ResourcesService::list_tags(&pool, id).await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tag_name, "web");
}

#[tokio::test]
async fn set_tags_rejects_unknown_tag() {
    let pool = fresh().await;
    let id = ResourcesService::create(&pool, "Bob", None).await.unwrap();
    let err = ResourcesService::set_tags(&pool, id, &[999]).await.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

fn null() -> Option<&'static str> { None }
