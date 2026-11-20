use super::{blocking, ManagerState};
use crate::{models::ScheduleInput, storage::Result};

#[tauri::command]
// IPC: 예약 검증·저장.
pub async fn save_schedule(
    manager: ManagerState<'_>,
    id: Option<String>,
    input: ScheduleInput,
) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.upsert(id, input)).await
}

#[tauri::command]
// IPC: 예약 동작 처리.
pub async fn schedule_action(manager: ManagerState<'_>, id: String, action: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.action(id, &action)).await
}
