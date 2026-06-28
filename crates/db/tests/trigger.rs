use db::pool::connect;

async fn setup() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-06-30')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-10','2026-06-20')")
        .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn in_window_allocation_ok() {
    let pool = setup().await;
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-12','2026-06-18',0.5)"
    ).execute(&pool).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn out_of_task_window_rejected() {
    let pool = setup().await;
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-05','2026-06-18',0.5)"
    ).execute(&pool).await;
    assert!(res.is_err(), "allocation starting before task window must be aborted by trigger");
}

#[tokio::test]
async fn out_of_resource_availability_rejected() {
    let pool = setup().await;
    sqlx::query("UPDATE resources SET available_from='2026-06-15', available_to='2026-06-25' WHERE id=1")
        .execute(&pool).await.unwrap();
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-10','2026-06-20',0.5)"
    ).execute(&pool).await;
    assert!(res.is_err(), "allocation outside resource availability must be aborted");
}