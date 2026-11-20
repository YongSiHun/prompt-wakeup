use crate::{
    models::*,
    security::{require, SecurityValidator},
    storage::Result,
};
pub struct ScheduleService;
impl ScheduleService {
    pub fn validate(input: &ScheduleInput) -> Result<()> {
        SecurityValidator::input(input, true)?;
        crate::ai_cli::adapter(input.cli).get_version(&input.cli_path)?;
        Ok(())
    }
    pub fn upsert(
        state: &mut Snapshot,
        id: Option<String>,
        mut input: ScheduleInput,
    ) -> Result<()> {
        input.name = input.name.trim().to_string();
        require(!input.name.is_empty(), "작업 이름을 입력하세요.")?;
        require(
            !state.schedules.iter().any(|schedule| {
                Some(&schedule.id) != id.as_ref() && schedule.input.name.trim() == input.name
            }),
            "이미 사용 중인 예약 작업 이름입니다. 다른 이름을 입력하세요.",
        )?;
        match id {
            Some(id) => Self::update(state, &id, input),
            None => {
                let highest = state
                    .schedules
                    .iter()
                    .filter_map(|schedule| {
                        schedule
                            .input
                            .name
                            .trim()
                            .strip_prefix("새 예약 작업 ")
                            .and_then(|number| number.parse::<u64>().ok())
                    })
                    .max()
                    .unwrap_or(0);
                state.schedule_sequence = state
                    .schedule_sequence
                    .max(highest)
                    .max(state.schedules.len() as u64)
                    + 1;
                state.schedules.push(Schedule {
                    id: uuid::Uuid::new_v4().to_string(),
                    input,
                    status: ScheduleStatus::Pending,
                });
                Ok(())
            }
        }
    }
    fn update(state: &mut Snapshot, id: &str, input: ScheduleInput) -> Result<()> {
        let schedule = Self::find(state, id)?;
        require(
            schedule.status == ScheduleStatus::Pending,
            "대기 중인 예약만 수정할 수 있습니다.",
        )?;
        schedule.input = input;
        Ok(())
    }
    pub fn find<'a>(state: &'a mut Snapshot, id: &str) -> Result<&'a mut Schedule> {
        state
            .schedules
            .iter_mut()
            .find(|schedule| schedule.id == id)
            .ok_or_else(|| "예약을 찾을 수 없습니다.".into())
    }
    pub fn action(state: &mut Snapshot, id: &str, action: &str) -> Result<()> {
        let schedule = Self::find(state, id)?;
        require(
            !schedule.status.is_running(),
            "실행 중인 작업은 변경할 수 없습니다.",
        )?;
        match action {
            "cancel" => Self::cancel(schedule),
            "delete" => {
                state.schedules.retain(|schedule| schedule.id != id);
                Ok(())
            }
            _ => Err("지원하지 않는 작업".into()),
        }
    }
    fn cancel(schedule: &mut Schedule) -> Result<()> {
        require(
            schedule.status == ScheduleStatus::Pending,
            "대기 중인 예약만 취소할 수 있습니다.",
        )?;
        schedule.status = ScheduleStatus::Canceled;
        Ok(())
    }
    pub fn validate_launch(schedule: &Schedule, manual: bool) -> Result<()> {
        require(!schedule.status.is_running(), "이미 실행 중인 예약입니다.")?;
        require(
            manual || Self::is_due(schedule, chrono::Utc::now()),
            "이미 처리했거나 실행 시각이 변경된 예약입니다.",
        )
    }
    pub fn is_due(schedule: &Schedule, now: chrono::DateTime<chrono::Utc>) -> bool {
        schedule.status == ScheduleStatus::Pending && schedule.input.scheduled_at <= now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_snapshot() -> Snapshot {
        Snapshot {
            schedule_sequence: 0,
            schedules: Vec::new(),
            executions: Vec::new(),
            settings: Settings::default(),
            active: true,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn unique_names() {
        let directory = tempfile::tempdir().unwrap();
        let mut state = empty_snapshot();
        let mut input = crate::test_support::input(directory.path());
        input.name = "새 예약 작업 1".into();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        input.name = " 새 예약 작업 1 ".into();
        assert!(ScheduleService::upsert(&mut state, None, input.clone()).is_err());
        assert_eq!(state.schedule_sequence, 1);
        let id = state.schedules[0].id.clone();
        ScheduleService::upsert(&mut state, Some(id.clone()), input.clone()).unwrap();
        ScheduleService::action(&mut state, &id, "cancel").unwrap();
        assert!(ScheduleService::upsert(&mut state, None, input.clone()).is_err());
        input.name = "새 예약 작업 2".into();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        let second = state.schedules[1].id.clone();
        input.name = "새 예약 작업 1".into();
        assert!(ScheduleService::upsert(&mut state, Some(second.clone()), input.clone()).is_err());
        input.name = "사용자 이름".into();
        ScheduleService::upsert(&mut state, Some(second.clone()), input.clone()).unwrap();
    }

    #[test]
    fn monotonic_sequence_after_deletion_and_restore() {
        let directory = tempfile::tempdir().unwrap();
        let mut state = empty_snapshot();
        let mut input = crate::test_support::input(directory.path());
        input.name = "새 예약 작업 1".into();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        let id = state.schedules[0].id.clone();
        input.name = "새 예약 작업 2".into();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        let second = state.schedules[1].id.clone();
        input.name = "사용자 이름".into();
        ScheduleService::upsert(&mut state, Some(second.clone()), input.clone()).unwrap();
        ScheduleService::action(&mut state, &id, "delete").unwrap();
        ScheduleService::action(&mut state, &second, "delete").unwrap();
        let serialized = serde_json::to_string(&state).unwrap();
        let mut restored: Snapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.schedule_sequence, 2);
        ScheduleService::upsert(&mut restored, None, input).unwrap();
        assert_eq!(restored.schedule_sequence, 3);
    }
}
