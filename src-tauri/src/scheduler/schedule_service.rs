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
    pub fn upsert(state: &mut Snapshot, id: Option<String>, input: ScheduleInput) -> Result<()> {
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
