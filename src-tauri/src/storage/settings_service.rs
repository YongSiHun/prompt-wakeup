use crate::{models::{CliKind, Settings}, security::require, storage::Result};
pub struct SettingsService;
impl SettingsService {
    pub fn validate(settings: &Settings) -> Result<()> {
        require(["system", "light", "dark"].contains(&settings.theme.as_str()), "설정 테마가 올바르지 않습니다.")?;
        require((1..=3650).contains(&settings.retention_days), "보존 기간은 1~3650일이어야 합니다.")?;
        require(settings.custom_terminals.len() <= 100, "사용자 지정 터미널은 최대 100개입니다.")?;
        require(settings.custom_cli_paths.keys().all(|key| CliKind::ALL.iter().any(|kind| kind.name() == key)), "사용자 지정 CLI 종류가 올바르지 않습니다.")
    }
}
