use super::{blocking, ManagerState};
use crate::storage::{output_files, Result};

#[tauri::command]
// IPC: 완료 이력·출력 삭제.
pub async fn delete_execution(manager: ManagerState<'_>, id: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.delete_execution(&id)).await
}

#[tauri::command]
// IPC: 최근 출력 조회.
pub async fn execution_output(manager: ManagerState<'_>, id: String) -> Result<String> {
    let repository = manager.repository.clone();
    blocking(move || {
        let path = repository.run_path(&id, "output")?;
        output_files::read_preview(&path)
    })
    .await
}

#[tauri::command]
// IPC: 검증된 출력 경로.
pub fn execution_output_path(manager: ManagerState<'_>, id: String) -> Result<String> {
    Ok(manager
        .repository
        .run_path(&id, "output")?
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
// IPC: 출력 파일·폴더 열기.
pub async fn open_execution_output(
    manager: ManagerState<'_>,
    id: String,
    folder: bool,
) -> Result<()> {
    let path = manager.repository.run_path(&id, "output")?;
    blocking(move || crate::storage::output_files::open(&path, folder)).await
}
