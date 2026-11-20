pub mod app;
pub mod clis;
pub mod executions;
pub mod schedules;
pub mod settings;

use crate::{runtime::blocking, scheduler::SchedulerManager};
use std::sync::Arc;
use tauri::State;

type ManagerState<'a> = State<'a, Arc<SchedulerManager>>;
