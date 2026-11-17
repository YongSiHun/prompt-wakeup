use super::{blocking, ManagerState};
use crate::{models::ScheduleInput, storage::Result};

#[tauri::command]
pub async fn save_schedule(
    manager: ManagerState<'_>,
    id: Option<String>,
    input: ScheduleInput,
) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.upsert(id, input)).await
}

#[tauri::command]
pub async fn schedule_action(manager: ManagerState<'_>, id: String, action: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.action(id, &action)).await
}
