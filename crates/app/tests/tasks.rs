use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use app::service::catalog::CatalogService;
use db::pool::connect;

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn create_task_with_skills_and_tags_atomic() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let urgent = CatalogService::ensure_tag(&pool, "urgent", None).await.unwrap();

    let tid = TasksService::create(
        &pool, pid, "Build API", None, 5.0, Some("2026-06-01"), Some("2026-06-10"),
        false, None, None, 0, &[(rust, 3, true, 1.0)], &[urgent]).await.unwrap();

    let reqs = TasksService::skill_reqs(&pool, tid).await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].skill_id, rust);
    assert_eq!(reqs[0].min_proficiency, 3);

    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb.len(), 1);
    assert_eq!(kb[0].title, "Build API");
    assert_eq!(kb[0].skill_count, 1);
    assert_eq!(kb[0].assignee, None); // no allocation yet
}

#[tokio::test]
async fn invalid_proficiency_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let err = TasksService::create(
        &pool, pid, "T", None, 1.0, None, None, false, None, None, 0, &[(1, 9, true, 1.0)], &[]).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn set_status_transitions() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let tid = TasksService::create(&pool, pid, "T", None, 1.0, None, None, false, None, None, 0, &[], &[]).await.unwrap();
    TasksService::set_status(&pool, tid, "in_progress").await.unwrap();
    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb[0].status, "in_progress");
    let err = TasksService::set_status(&pool, tid, "bogus").await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn task_create_rolls_back_on_child_failure() {
    // A skill_id that doesn't exist must violate the FK and abort the whole tx —
    // no orphan task row should remain. This verifies with_write_tx rollback.
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();

    let res = TasksService::create(
        &pool, pid, "Orphan", None, 1.0, None, None, false, None, None, 0,
        &[(9999, 3, true, 1.0)], &[]).await; // skill_id 9999 does not exist
    assert!(res.is_err(), "FK violation on skill_req must fail");

    // No task row should survive the rollback.
    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert!(kb.is_empty(), "task row must not persist after child-insert failure");
}

// ---- P2 #1: long-term task + segmented scheduling (design §3.4) ----

#[tokio::test]
async fn long_term_task_and_segment_persist() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    // Long-term parent task.
    let parent = TasksService::create(
        &pool, pid, "Epic", None, 80.0, Some("2026-06-01"), Some("2026-09-30"),
        true, None, None, 0, &[], &[],
    ).await.unwrap();
    // A segment under the parent.
    let seg = TasksService::create(
        &pool, pid, "Phase 1", None, 20.0, Some("2026-06-01"), Some("2026-06-20"),
        false, Some(parent), Some("phase"), 1, &[], &[],
    ).await.unwrap();

    let tasks = TasksService::list_by_project(&pool, pid).await.unwrap();
    let p = tasks.iter().find(|t| t.id == parent).unwrap();
    assert_eq!(p.is_long_term, 1);
    assert!(p.parent_task_id.is_none());
    let s = tasks.iter().find(|t| t.id == seg).unwrap();
    assert_eq!(s.parent_task_id, Some(parent));
    assert_eq!(s.segment_kind.as_deref(), Some("phase"));
}

#[tokio::test]
async fn segment_without_parent_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    // segment_kind set but no parent_task_id ⇒ invalid (design §3.4: a segment belongs to a parent).
    let err = TasksService::create(
        &pool, pid, "Orphan segment", None, 5.0, None, None,
        false, None, Some("phase"), 0, &[], &[],
    ).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn invalid_segment_kind_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let parent = TasksService::create(
        &pool, pid, "Epic", None, 80.0, None, None, true, None, None, 0, &[], &[]).await.unwrap();
    // segment_kind must be milestone|phase|segment.
    let err = TasksService::create(
        &pool, pid, "Bad", None, 5.0, None, None,
        false, Some(parent), Some("bogus"), 0, &[], &[],
    ).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn segment_parent_must_exist_in_same_project() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    // parent_task_id references a non-existent task.
    let err = TasksService::create(
        &pool, pid, "Seg", None, 5.0, None, None,
        false, Some(9999), Some("segment"), 0, &[], &[],
    ).await.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[tokio::test]
async fn update_to_segment_with_cycle_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, true, None, None, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, true, None, None, 1, &[], &[]).await.unwrap();
    // B's parent = A (ok).
    TasksService::update(&pool, b, "B", None, 1.0, None, None, true, Some(a), None).await.unwrap();
    // Now set A's parent = B ⇒ cycle, must be rejected.
    let err = TasksService::update(&pool, a, "A", None, 1.0, None, None, true, Some(b), None)
        .await.unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

#[tokio::test]
async fn update_self_parent_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, true, None, None, 0, &[], &[]).await.unwrap();
    let err = TasksService::update(&pool, a, "A", None, 1.0, None, None, true, Some(a), None)
        .await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}