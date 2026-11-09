pub mod settings_service;
use crate::models::Snapshot;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs::{self, OpenOptions}, io::Write, path::{Path, PathBuf}};

pub type Result<T> = std::result::Result<T, String>;
const SCHEMA_VERSION: u32 = 1;
#[derive(Serialize, Deserialize)]
struct Envelope<T> { schema_version: u32, data: T }
enum LoadError { Schema, Corrupt(String) }
#[derive(Clone)]
pub struct StorageRepository { pub root: PathBuf }

impl StorageRepository {
    pub fn open() -> Result<Self> {
        let root = dirs::data_local_dir().ok_or("사용자 데이터 경로를 찾지 못했습니다.")?.join("PromptWakeup");
        Self::at(root)
    }
    pub fn at(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root).map_err(err)?;
        restrict(&root)?;
        fs::create_dir_all(root.join("runs")).map_err(err)?;
        Ok(Self { root })
    }
    pub fn load<T: DeserializeOwned + Default>(&self, name: &str, warnings: &mut Vec<String>) -> Result<T> {
        let path = self.root.join(name);
        if !path.exists() { return Ok(T::default()); }
        match read(&path) {
            Ok(value) => Ok(value),
            Err(LoadError::Schema) => Err(schema_error()),
            Err(LoadError::Corrupt(error)) => self.restore(name, error, warnings),
        }
    }
    fn restore<T: DeserializeOwned>(&self, name: &str, error: String, warnings: &mut Vec<String>) -> Result<T> {
        let path = self.root.join(name);
        let backup = path.with_extension("json.bak");
        let value = read(&backup).map_err(|_| format!("{name} 손상: {error}. 원본을 보존했습니다. 데이터 폴더에서 복구하세요."))?;
        fs::copy(&path, path.with_extension(format!("corrupt-{}", uuid::Uuid::new_v4()))).map_err(err)?;
        fs::copy(backup, &path).map_err(err)?;
        warnings.push(format!("{name} 손상으로 백업을 복구했습니다."));
        Ok(value)
    }
    pub fn save<T: Serialize>(&self, name: &str, data: &T) -> Result<()> {
        let path = self.root.join(name);
        backup(&path)?;
        atomic(&path, &Envelope { schema_version: SCHEMA_VERSION, data })
    }
    pub fn save_changes(&self, previous: &Snapshot, next: &Snapshot) -> Result<()> {
        // Commit history before schedules so recovery can detect interrupted launches.
        if previous.executions != next.executions { self.save("executions.json", &next.executions)?; }
        if previous.schedules != next.schedules { self.save("schedules.json", &next.schedules)?; }
        if previous.settings != next.settings { self.save("settings.json", &next.settings)?; }
        Ok(())
    }
    pub fn run_path(&self, id: &str, suffix: &str) -> Result<PathBuf> {
        uuid::Uuid::parse_str(id).map_err(|_| "실행 ID가 올바르지 않습니다.")?;
        crate::security::require(["ticket", "claim", "result"].contains(&suffix), "실행 데이터 종류가 올바르지 않습니다.")?;
        Ok(self.root.join("runs").join(format!("{id}.{suffix}")))
    }
    pub fn remove_run_file(&self, id: &str, suffix: &str) {
        if let Ok(path) = self.run_path(id, suffix) { let _ = fs::remove_file(path); }
    }
    pub fn remove_run(&self, id: &str) {
        for suffix in ["ticket", "claim", "result"] { self.remove_run_file(id, suffix); }
    }
    pub fn key(&self, create: bool) -> Result<Vec<u8>> {
        let path = self.root.join("runner.key");
        if path.exists() { return read_key(&path); }
        crate::security::require(create, "실행 키가 없습니다.")?;
        create_key(&path)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let mut warnings = Vec::new();
        Ok(Snapshot {
            schedules: self.load("schedules.json", &mut warnings)?,
            executions: self.load("executions.json", &mut warnings)?,
            settings: self.load("settings.json", &mut warnings)?, active: true, warnings,
        })
    }
}
fn read<T: DeserializeOwned>(path: &Path) -> std::result::Result<T, LoadError> {
    let bytes = fs::read(path).map_err(|error| LoadError::Corrupt(err(error)))?;
    let data: Envelope<T> = serde_json::from_slice(&bytes).map_err(|error| LoadError::Corrupt(err(error)))?;
    if data.schema_version != SCHEMA_VERSION { return Err(LoadError::Schema); }
    Ok(data.data)
}
fn schema_error() -> String { "지원하지 않는 스키마입니다. 원본을 보존하고 앱을 업데이트하세요.".into() }
fn backup(path: &Path) -> Result<()> {
    if path.exists() { fs::copy(path, path.with_extension("json.bak")).map_err(err)?; }
    Ok(())
}
fn read_key(path: &Path) -> Result<Vec<u8>> {
    let key = fs::read(path).map_err(err)?;
    crate::security::require(key.len() == 32, "실행 키가 손상되었습니다.")?;
    Ok(key)
}
fn create_key(path: &Path) -> Result<Vec<u8>> {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(err)?;
    file.write_all(&key).map_err(err)?;
    file.sync_all().map_err(err)?;
    Ok(key.to_vec())
}
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
fn sync_directory(path: &Path) -> Result<()> { fs::File::open(path).and_then(|file| file.sync_all()).map_err(err) }
#[cfg(not(unix))]
fn sync_directory(_: &Path) -> Result<()> { Ok(()) }
pub fn err(error: impl std::fmt::Display) -> String { format!("파일 또는 프로세스 처리 실패: {error}") }
#[cfg(unix)]
fn restrict(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(err)
}
#[cfg(windows)]
fn restrict(path: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    let user = format!("{}\\{}", std::env::var("USERDOMAIN").map_err(err)?, std::env::var("USERNAME").map_err(err)?);
    let output = std::process::Command::new("icacls.exe").arg(path)
        .args(["/inheritance:r", "/grant:r", &format!("{user}:(OI)(CI)F")])
        .creation_flags(0x08000000).output().map_err(err)?;
    crate::security::require(output.status.success(), "데이터 폴더 접근 권한 제한에 실패했습니다.")
}
