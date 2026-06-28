use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn migration_creates_all_tables() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let expected = [
        "settings","tags","skills","resources","resource_skills","resource_tags",
        "teams","team_members","team_overrides","work_week_template","holiday","time_off",
        "projects","tasks","task_dependencies","task_skill_requirements","task_tags",
        "allocations","ai_optimization_runs","resource_project_rates",
    ];
    for tbl in expected {
        let exists: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(tbl)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(exists.0, 1, "table {} missing after migration", tbl);
    }

    // seeded single settings row + global work-week template
    let n_settings: (i64,) = sqlx::query_as("SELECT count(*) FROM settings").fetch_one(&pool).await.unwrap();
    assert_eq!(n_settings.0, 1);
    let n_week: (i64,) = sqlx::query_as("SELECT count(*) FROM work_week_template WHERE scope='global'").fetch_one(&pool).await.unwrap();
    assert_eq!(n_week.0, 1);
}

#[tokio::test]
async fn global_workweek_unique_constraint() {
    let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // inserting a second global row must fail (idx_wwt_global)
    let res = sqlx::query(
        "INSERT INTO work_week_template (scope) VALUES ('global')"
    ).execute(&pool).await;
    assert!(res.is_err(), "second global work_week_template should be rejected");
}