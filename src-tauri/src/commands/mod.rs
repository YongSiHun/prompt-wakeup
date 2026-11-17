pub mod app;
pub mod clis;
pub mod executions;
pub mod schedules;
pub mod settings;

use crate::{
    scheduler::SchedulerManager,
    storage::{err, Result},
};
use std::sync::Arc;
use tauri::State;

type ManagerState<'a> = State<'a, Arc<SchedulerManager>>;
async fn blocking<T, F>(work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(err)?
}
