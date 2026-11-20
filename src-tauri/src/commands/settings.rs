use super::{blocking, ManagerState};
use crate::{models::Settings, storage::Result};

#[tauri::command]
// IPC: 설정 검증·저장.
pub async fn save_settings(manager: ManagerState<'_>, settings: Settings) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.settings(settings)).await
}
