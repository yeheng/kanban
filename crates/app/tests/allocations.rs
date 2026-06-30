use app::service::allocations::AllocationsService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use db::pool::connect;

async fn seeded() -> (sqlx::SqlitePool, i64) {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0)
        .await
        .unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool)
        .await
        .unwrap();
    let task_id = TasksService::create(
        &pool,
        pid,
        "T",
        None,
        1.0,
        Some("2026-06-01"),
        Some("2026-06-10"),
        false,
        None,
        None,
        0,
        &[],
        &[],
    )
    .await
    .unwrap();
    (pool, task_id)
}

#[tokio::test]
async fn out_of_task_window_allocation_maps_to_domain_not_db() {
    // The task window is 2026-06-01..2026-06-10; an allocation starting before it is rejected
    // by the DB trigger (RAISE ABORT). That is a user-correctable business-rule violation, so
    // it must surface as a 4xx DOMAIN error, not a DB 500.
    let (pool, task_id) = seeded().await;
    let err = AllocationsService::create(&pool, 1, task_id, "2026-05-20", "2026-06-05", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN", "trigger rejection should map to DOMAIN, got {:?}", err);
}

#[tokio::test]
async fn create_rejects_malformed_dates() {
    let (pool, task_id) = seeded().await;
    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-99", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");

    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-1", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn update_rejects_bad_window_before_repo() {
    let (pool, task_id) = seeded().await;
    let id = AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
    let err = AllocationsService::update(&pool, id, "2026-06-06", "2026-06-05", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn overlapping_allocations_over_capacity_are_blocked() {
    let (pool, task_id) = seeded().await;
    let task2 = TasksService::create(
        &pool,
        1,
        "T2",
        None,
        1.0,
        Some("2026-06-01"),
        Some("2026-06-10"),
        false,
        None,
        None,
        0,
        &[],
        &[],
    )
    .await
    .unwrap();
    AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.7)
        .await
        .unwrap();
    let err = AllocationsService::create(&pool, 1, task2, "2026-06-03", "2026-06-06", 0.4)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

#[tokio::test]
async fn weekend_only_overlap_does_not_count_as_capacity_conflict() {
    let (pool, task_id) = seeded().await;
    let task2 = TasksService::create(
        &pool,
        1,
        "T2",
        None,
        1.0,
        Some("2026-06-01"),
        Some("2026-06-10"),
        false,
        None,
        None,
        0,
        &[],
        &[],
    )
    .await
    .unwrap();
    AllocationsService::create(&pool, 1, task_id, "2026-06-05", "2026-06-07", 0.8)
        .await
        .unwrap();
    AllocationsService::create(&pool, 1, task2, "2026-06-07", "2026-06-08", 0.8)
        .await
        .unwrap();
}

#[tokio::test]
async fn dependency_order_is_blocked_for_allocations() {
    let (pool, predecessor) = seeded().await;
    let successor = TasksService::create(
        &pool,
        1,
        "T2",
        None,
        1.0,
        Some("2026-06-01"),
        Some("2026-06-10"),
        false,
        None,
        None,
        0,
        &[],
        &[],
    )
    .await
    .unwrap();
    AllocationsService::create(&pool, 1, predecessor, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
    TasksService::add_dependency(&pool, successor, predecessor, 0, "FS")
        .await
        .unwrap();
    let err = AllocationsService::create(&pool, 1, successor, "2026-06-04", "2026-06-10", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

// ---- P1: mandatory skill enforcement (design §3.8 #4, §9 #12) ----

/// Seed a task with mandatory skill requirements and a resource with skills.
async fn seeded_skills(
    task_reqs: &[(i64 /*skill_id*/, i64 /*min_prof*/, bool /*mandatory*/)],
    resource_skills: &[(i64 /*skill_id*/, i64 /*prof*/)],
) -> (sqlx::SqlitePool, i64 /*task_id*/) {
    let (pool, task_id) = seeded().await;
    // Ensure skill dictionary rows exist for all referenced skill_ids (FK target).
    let mut skill_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for &(sid, _, _) in task_reqs {
        skill_ids.insert(sid);
    }
    for &(sid, _) in resource_skills {
        skill_ids.insert(sid);
    }
    for sid in skill_ids {
        sqlx::query("INSERT OR IGNORE INTO skills (id, name) VALUES (?, ?)")
            .bind(sid)
            .bind(format!("skill-{}", sid))
            .execute(&pool)
            .await
            .unwrap();
    }
    for &(sid, min_prof, mandatory) in task_reqs {
        sqlx::query(
            "INSERT INTO task_skill_requirements (task_id, skill_id, min_proficiency, is_mandatory, weight) \
             VALUES (?,?,?,?,1.0)",
        )
        .bind(task_id)
        .bind(sid)
        .bind(min_prof)
        .bind(mandatory as i64)
        .execute(&pool)
        .await
        .unwrap();
    }
    for &(sid, prof) in resource_skills {
        sqlx::query("INSERT INTO resource_skills (resource_id, skill_id, proficiency) VALUES (1,?,?)")
            .bind(sid)
            .bind(prof)
            .execute(&pool)
            .await
            .unwrap();
    }
    (pool, task_id)
}

#[tokio::test]
async fn allocation_allowed_when_mandatory_skills_satisfied() {
    // skill 1 min-prof 3 mandatory; Alice holds proficiency 4.
    let (pool, task_id) = seeded_skills(&[(1, 3, true)], &[(1, 4)]).await;
    AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
}

#[tokio::test]
async fn allocation_blocked_when_skill_missing() {
    // Mandatory skill 1 required; Alice holds none.
    let (pool, task_id) = seeded_skills(&[(1, 3, true)], &[]).await;
    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

#[tokio::test]
async fn allocation_blocked_when_proficiency_insufficient() {
    // Mandatory skill 1 min-prof 4; Alice holds only proficiency 2.
    let (pool, task_id) = seeded_skills(&[(1, 4, true)], &[(1, 2)]).await;
    let err = AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}

#[tokio::test]
async fn soft_skill_requirement_does_not_block() {
    // Non-mandatory skill requirement: missing skill should NOT block allocation.
    let (pool, task_id) = seeded_skills(&[(1, 5, false)], &[]).await;
    AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
}

#[tokio::test]
async fn allocation_update_also_enforces_skills() {
    let (pool, task_id) = seeded_skills(&[(1, 3, true)], &[(1, 4)]).await;
    let id = AllocationsService::create(&pool, 1, task_id, "2026-06-01", "2026-06-05", 0.5)
        .await
        .unwrap();
    // Remove Alice's skill, then updating the allocation must fail.
    sqlx::query("DELETE FROM resource_skills WHERE resource_id = 1 AND skill_id = 1")
        .execute(&pool)
        .await
        .unwrap();
    let err = AllocationsService::update(&pool, id, "2026-06-02", "2026-06-06", 0.5)
        .await
        .unwrap_err();
    assert_eq!(err.code, "DOMAIN");
}
