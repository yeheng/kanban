use db::pool::connect;
use db::repo::gantt::GanttRepo;

#[tokio::test]
async fn by_project_and_cross_project_by_resource() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'Atlas')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'Borealis')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T1','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,2,'T2','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-08','2026-06-12',0.5)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,11,'2026-06-15','2026-06-19',0.3)").execute(&pool).await.unwrap();

    // project view: only Atlas's allocation
    let p1 = GanttRepo::by_project(&pool, 1).await.unwrap();
    assert_eq!(p1.len(), 1);
    assert_eq!(p1[0].project_name, "Atlas");
    assert_eq!(p1[0].resource_name, "Alice");

    // cross-project resource view: both, different projects
    let r1 = GanttRepo::by_resource(&pool, 1).await.unwrap();
    assert_eq!(r1.len(), 2);
    let names: Vec<_> = r1.iter().map(|b| b.project_name.as_str()).collect();
    assert!(names.contains(&"Atlas") && names.contains(&"Borealis"));
}
