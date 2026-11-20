pub mod output_files;
pub mod settings_service;
use crate::models::Snapshot;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, String>;
const SCHEMA_VERSION: u32 = 1;
#[derive(Serialize, Deserialize)]
struct Envelope<T> {
    schema_version: u32,
    data: T,
}
enum LoadError {
    Schema,
    Corrupt(String),
}
#[derive(Clone)]
pub struct StorageRepository {
    pub root: PathBuf,
}

impl StorageRepository {
    // I/O: 저장소·파일 열기.
    pub fn open() -> Result<Self> {
        let root = dirs::data_local_dir()
            .ok_or("사용자 데이터 경로를 찾지 못했습니다.")?
            .join("PromptWakeup");
        Self::at(root)
    }
    // I/O: 저장소 초기화·접근 제한.
    pub fn at(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root).map_err(err)?;
        restrict(&root)?;
        fs::create_dir_all(root.join("runs")).map_err(err)?;
        Ok(Self { root })
    }
    // I/O: 데이터·백업 복구.
    pub fn load<T: DeserializeOwned + Default>(
        &self,
        name: &str,
        warnings: &mut Vec<String>,
    ) -> Result<T> {
        let path = self.root.join(name);
        if !path.exists() {
            return Ok(T::default());
        }
        match read(&path) {
            Ok(value) => Ok(value),
            Err(LoadError::Schema) => Err(schema_error()),
            Err(LoadError::Corrupt(error)) => self.restore(name, error, warnings),
        }
    }
    // I/O: 손상 원본 보존·백업 복구.
    fn restore<T: DeserializeOwned>(
        &self,
        name: &str,
        error: String,
        warnings: &mut Vec<String>,
    ) -> Result<T> {
        let path = self.root.join(name);
        let backup = path.with_extension("json.bak");
        let value = read(&backup).map_err(|_| {
            format!("{name} 손상: {error}. 원본을 보존했습니다. 데이터 폴더에서 복구하세요.")
        })?;
        fs::copy(
            &path,
            path.with_extension(format!("corrupt-{}", uuid::Uuid::new_v4())),
        )
        .map_err(err)?;
        fs::copy(backup, &path).map_err(err)?;
        warnings.push(format!("{name} 손상으로 백업을 복구했습니다."));
        Ok(value)
    }
    // I/O: 백업·원자적 저장.
    pub fn save<T: Serialize>(&self, name: &str, data: &T) -> Result<()> {
        let path = self.root.join(name);
        backup(&path)?;
        atomic(
            &path,
            &Envelope {
                schema_version: SCHEMA_VERSION,
                data,
            },
        )
    }
    // I/O: 변경된 데이터만 저장.
    pub fn save_changes(&self, previous: &Snapshot, next: &Snapshot) -> Result<()> {
        // Commit history before schedules so recovery can detect interrupted launches.
        if previous.executions != next.executions {
            self.save("executions.json", &next.executions)?;
        }
        if previous.schedules != next.schedules {
            self.save("schedules.json", &next.schedules)?;
        }
        if previous.settings != next.settings {
            self.save("settings.json", &next.settings)?;
        }
        Ok(())
    }
    // 보안: UUID·파일 종류 검증.
    pub fn run_path(&self, id: &str, suffix: &str) -> Result<PathBuf> {
        uuid::Uuid::parse_str(id).map_err(|_| "실행 ID가 올바르지 않습니다.")?;
        crate::security::require(
            ["ticket", "claim", "result", "output"].contains(&suffix),
            "실행 데이터 종류가 올바르지 않습니다.",
        )?;
        Ok(self.root.join("runs").join(format!("{id}.{suffix}")))
    }
    // I/O: 실행 파일 정리.
    pub fn remove_run_file(&self, id: &str, suffix: &str) {
        if let Ok(path) = self.run_path(id, suffix) {
            let _ = fs::remove_file(path);
        }
    }
    // I/O: 실행 관련 파일 정리.
    pub fn remove_run(&self, id: &str) {
        for suffix in ["ticket", "claim", "result", "output"] {
            self.remove_run_file(id, suffix);
        }
    }
    // 보안: 실행 서명 키 조회·생성.
    pub fn key(&self, create: bool) -> Result<Vec<u8>> {
        let path = self.root.join("runner.key");
        if path.exists() {
            return read_key(&path);
        }
        crate::security::require(create, "실행 키가 없습니다.")?;
        create_key(&path)
    }
    // 조회: 현재 상태 복제.
    pub fn snapshot(&self) -> Result<Snapshot> {
        let mut warnings = Vec::new();
        let settings = self.load("settings.json", &mut warnings)?;
        settings_service::SettingsService::validate(&settings)?;
        Ok(Snapshot {
            schedules: self.load("schedules.json", &mut warnings)?,
            executions: self.load("executions.json", &mut warnings)?,
            settings,
            active: true,
            warnings,
        })
    }
}
// I/O: 저장 데이터 읽기.
fn read<T: DeserializeOwned>(path: &Path) -> std::result::Result<T, LoadError> {
    let bytes = fs::read(path).map_err(|error| LoadError::Corrupt(err(error)))?;
    let data: Envelope<T> =
        serde_json::from_slice(&bytes).map_err(|error| LoadError::Corrupt(err(error)))?;
    if data.schema_version != SCHEMA_VERSION {
        return Err(LoadError::Schema);
    }
    Ok(data.data)
}
// 검증: 미지원 스키마 안내.
fn schema_error() -> String {
    "지원하지 않는 스키마입니다. 원본을 보존하고 앱을 업데이트하세요.".into()
}
// I/O: 이전 파일 보존.
fn backup(path: &Path) -> Result<()> {
    if path.exists() {
        fs::copy(path, path.with_extension("json.bak")).map_err(err)?;
    }
    Ok(())
}
// 보안: 키 길이 검증.
fn read_key(path: &Path) -> Result<Vec<u8>> {
    let key = fs::read(path).map_err(err)?;
    crate::security::require(key.len() == 32, "실행 키가 손상되었습니다.")?;
    Ok(key)
}
// 보안: 무작위 키 생성·저장.
fn create_key(path: &Path) -> Result<Vec<u8>> {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(err)?;
    file.write_all(&key).map_err(err)?;
    file.sync_all().map_err(err)?;
    Ok(key.to_vec())
}
// I/O: 임시 파일로 원자적 교체.
pub fn atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path.parent().ok_or("저장 경로 오류")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    serde_json::to_writer(&mut file, value).map_err(err)?;
    file.flush().map_err(err)?;
    file.as_file().sync_all().map_err(err)?;
    file.persist(path).map_err(err)?;
    sync_directory(parent)
}
#[cfg(unix)]
// I/O: 디렉터리 동기화.
fn sync_directory(path: &Path) -> Result<()> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(err)
}
#[cfg(not(unix))]
// I/O: 디렉터리 동기화.
fn sync_directory(_: &Path) -> Result<()> {
    Ok(())
}
// 변환: 파일·프로세스 오류.
pub fn err(error: impl std::fmt::Display) -> String {
    format!("파일 또는 프로세스 처리 실패: {error}")
}
#[cfg(unix)]
// 보안: 사용자 전용 접근 제한.
fn restrict(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(err)
}
#[cfg(windows)]
// 보안: 사용자 전용 접근 제한.
fn restrict(path: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    let user = current_user_sid()?;
    let system = std::env::var("SystemRoot").map_err(err)?;
    let output = std::process::Command::new(Path::new(&system).join("System32/icacls.exe"))
        .arg(path)
        .args(["/inheritance:r", "/grant:r", &format!("*{user}:(OI)(CI)F")])
        .creation_flags(0x08000000)
        .output()
        .map_err(err)?;
    crate::security::require(
        output.status.success(),
        "데이터 폴더 접근 권한 제한에 실패했습니다.",
    )
}

#[cfg(windows)]
// OS: 실제 프로세스 계정 SID.
fn current_user_sid() -> Result<String> {
    use std::os::windows::process::CommandExt;
    // Environment variables can describe a different account than the process token.
    let system = std::env::var("SystemRoot").map_err(err)?;
    let output = std::process::Command::new(Path::new(&system).join("System32/whoami.exe"))
        .args(["/user", "/fo", "csv", "/nh"])
        .creation_flags(0x08000000)
        .output()
        .map_err(err)?;
    crate::security::require(output.status.success(), "실행 계정 조회에 실패했습니다.")?;
    let identity = String::from_utf8_lossy(&output.stdout);
    let (_, sid) = identity
        .trim()
        .rsplit_once(',')
        .ok_or("실행 계정 SID가 없습니다.")?;
    let sid = sid.trim_matches('"');
    crate::security::require(
        sid.starts_with("S-1-")
            && sid[4..]
                .chars()
                .all(|value| value.is_ascii_digit() || value == '-'),
        "실행 계정 SID가 올바르지 않습니다.",
    )?;
    Ok(sid.to_owned())
}

#[cfg(test)]
mod tests;
