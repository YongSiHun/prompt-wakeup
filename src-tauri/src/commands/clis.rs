use super::{blocking, ManagerState};
use crate::{
    models::{CliInfo, CliKind},
    storage::Result,
};

#[tauri::command]
// IPC: CLI 탐지.
pub async fn detect_clis(manager: ManagerState<'_>) -> Result<Vec<CliInfo>> {
    let settings = manager.current_settings()?;
    blocking(move || Ok(crate::ai_cli::detect(&settings))).await
}

#[tauri::command]
// IPC: CLI 경로 검증·등록.
pub async fn manual_cli(manager: ManagerState<'_>, kind: CliKind, path: String) -> Result<CliInfo> {
    let manager = manager.inner().clone();
    blocking(move || {
        let info = crate::ai_cli::inspect(kind, path.clone());
        crate::security::require(
            info.available,
            info.validation_error
                .as_deref()
                .unwrap_or("CLI를 확인할 수 없습니다."),
        )?;
        manager.remember_cli(kind, path)?;
        Ok(info)
    })
    .await
}
