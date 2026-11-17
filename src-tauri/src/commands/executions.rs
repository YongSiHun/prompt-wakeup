use super::{blocking, ManagerState};
use crate::storage::{err, Result};

#[tauri::command]
pub async fn delete_execution(manager: ManagerState<'_>, id: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.delete_execution(&id)).await
}

#[tauri::command]
pub async fn execution_output(manager: ManagerState<'_>, id: String) -> Result<String> {
    let repository = manager.repository.clone();
    blocking(move || {
        let path = repository.run_path(&id, "output")?;
        match std::fs::read(path) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(error) => Err(err(error)),
        }
    })
    .await
}

#[tauri::command]
pub fn execution_output_path(manager: ManagerState<'_>, id: String) -> Result<String> {
    Ok(manager
        .repository
        .run_path(&id, "output")?
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub async fn open_execution_output(
    manager: ManagerState<'_>,
    id: String,
    folder: bool,
) -> Result<()> {
    let path = manager.repository.run_path(&id, "output")?;
    blocking(move || crate::storage::output_files::open(&path, folder)).await
}
