use crate::{
    models::*,
    security::{require, SecurityValidator},
    storage::Result,
};
pub struct ScheduleService;
impl ScheduleService {
    // 검증: 입력 정책.
    pub fn validate(input: &ScheduleInput) -> Result<()> {
        SecurityValidator::input(input, true)?;
        crate::ai_cli::version(&input.cli_path)?;
        Ok(())
    }
    // 상태: ID별 예약 저장.
    pub fn upsert(
        state: &mut Snapshot,
        id: Option<String>,
        mut input: ScheduleInput,
    ) -> Result<()> {
        input.name = input.name.trim().to_string();
        require(!input.name.is_empty(), "작업 이름을 입력하세요.")?;
        match id {
            Some(id) => Self::update(state, &id, input),
            None => {
                state.schedules.push(Schedule {
                    id: uuid::Uuid::new_v4().to_string(),
                    input,
                    status: ScheduleStatus::Pending,
                });
                Ok(())
            }
        }
    }
    // 상태: 대기 예약 수정.
    fn update(state: &mut Snapshot, id: &str, input: ScheduleInput) -> Result<()> {
        let schedule = Self::find(state, id)?;
        require(
            schedule.status == ScheduleStatus::Pending,
            "대기 중인 예약만 수정할 수 있습니다.",
        )?;
        schedule.input = input;
        Ok(())
    }
    // 조회: 이름·ID 검색.
    pub fn find<'a>(state: &'a mut Snapshot, id: &str) -> Result<&'a mut Schedule> {
        state
            .schedules
            .iter_mut()
            .find(|schedule| schedule.id == id)
            .ok_or_else(|| "예약을 찾을 수 없습니다.".into())
    }
    // 상태: 예약 동작 분기.
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
    // 상태: 대기 예약 취소.
    fn cancel(schedule: &mut Schedule) -> Result<()> {
        require(
            schedule.status == ScheduleStatus::Pending,
            "대기 중인 예약만 취소할 수 있습니다.",
        )?;
        schedule.status = ScheduleStatus::Canceled;
        Ok(())
    }
    // 검증: 중복·만료 예약 실행 차단.
    pub fn validate_launch(schedule: &Schedule, manual: bool) -> Result<()> {
        require(!schedule.status.is_running(), "이미 실행 중인 예약입니다.")?;
        require(
            manual || Self::is_due(schedule, chrono::Utc::now()),
            "이미 처리했거나 실행 시각이 변경된 예약입니다.",
        )
    }
    // 상태: 실행 시각 도달 여부.
    pub fn is_due(schedule: &Schedule, now: chrono::DateTime<chrono::Utc>) -> bool {
        schedule.status == ScheduleStatus::Pending && schedule.input.scheduled_at <= now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: empty snapshot.
    fn empty_snapshot() -> Snapshot {
        Snapshot {
            schedules: Vec::new(),
            executions: Vec::new(),
            settings: Settings::default(),
            active: true,
            warnings: Vec::new(),
        }
    }

    #[test]
    // Test: duplicate names are allowed with distinct ids.
    fn duplicate_names_are_allowed_with_distinct_ids() {
        let directory = tempfile::tempdir().unwrap();
        let mut state = empty_snapshot();
        let mut input = crate::test_support::input(directory.path());
        input.name = " 새 예약 작업 ".into();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        ScheduleService::upsert(&mut state, None, input.clone()).unwrap();
        assert_eq!(state.schedules[0].input.name, "새 예약 작업");
        assert_eq!(state.schedules[1].input.name, "새 예약 작업");
        assert_ne!(state.schedules[0].id, state.schedules[1].id);
        let id = state.schedules[1].id.clone();
        ScheduleService::upsert(&mut state, Some(id), input).unwrap();
    }
}
