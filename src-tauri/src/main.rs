mod commands_extra;

use app::state::AppState;
use commands_extra::*;
use std::env;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db_path = db_path_for(&handle);
                let state = match env::var("HRK_DB_PASSPHRASE") {
                    Ok(p) if !p.is_empty() => {
                        app::crypto::open_encrypted(&db_path, &p)
                            .await
                            .expect("failed to open encrypted DB")
                    }
                    _ => {
                        // Dev fallback: unencrypted file.
                        let url = format!("sqlite://{}?mode=rwc", db_path);
                        let pool =
                            db::pool::connect(&url).await.expect("dev db open");
                        sqlx::migrate!("../crates/db/migrations")
                            .run(&pool)
                            .await
                            .expect("migrate");
                        AppState { pool }
                    }
                };
                handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_project, list_projects,
            ensure_skill, list_skills, ensure_tag, list_tags,
            create_task, set_task_status, kanban_tasks,
            create_team, add_team_member, set_team_override,
            create_resource, list_resources,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn db_path_for(app: &tauri::AppHandle) -> String {
    let dir = app.path().app_data_dir().expect("app_data_dir");
    std::fs::create_dir_all(&dir).ok();
    dir.join("hrk.db").to_string_lossy().into_owned()
}