use crate::error::AppError;
use crate::state::AppState;

// Commands are added in Tasks 2–6. Each is #[tauri::command] delegating to a service.
#[allow(dead_code)]
fn _unused(_s: tauri::State<'_, AppState>) -> Result<(), AppError> { Ok(()) }