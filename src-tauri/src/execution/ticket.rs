use crate::{
    models::{Schedule, Ticket},
    security::{require, SecurityValidator},
    storage::{atomic, err, Result, StorageRepository},
};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[derive(serde::Serialize, serde::Deserialize)]
struct SignedTicket {
    payload: String,
    signature: String,
}
pub(super) fn prepare(repository: &StorageRepository, schedule: &Schedule, id: &str) -> Result<()> {
    SecurityValidator::input(&schedule.input, false)?;
    let ticket = Ticket {
        execution_id: id.into(),
        schedule: schedule.input.clone(),
        expires_at: Utc::now() + Duration::minutes(10),
    };
    let payload = serde_json::to_string(&ticket).map_err(err)?;
    let mut mac = new_mac(repository, true)?;
    mac.update(payload.as_bytes());
    let signed = SignedTicket {
        payload,
        signature: hex::encode(mac.finalize().into_bytes()),
    };
    atomic(&repository.run_path(id, "ticket")?, &signed)
}

fn new_mac(repository: &StorageRepository, create: bool) -> Result<Hmac<Sha256>> {
    Hmac::<Sha256>::new_from_slice(&repository.key(create)?).map_err(err)
}

pub(super) fn read(repository: &StorageRepository, id: &str) -> Result<Ticket> {
    let bytes = fs::read(repository.run_path(id, "ticket")?).map_err(err)?;
    let signed: SignedTicket = serde_json::from_slice(&bytes).map_err(err)?;
    let mut mac = new_mac(repository, false)?;
    mac.update(signed.payload.as_bytes());
    mac.verify_slice(&hex::decode(signed.signature).map_err(err)?)
        .map_err(|_| "실행 데이터 무결성 검증 실패")?;
    let ticket: Ticket = serde_json::from_str(&signed.payload).map_err(err)?;
    require(
        ticket.execution_id == id && ticket.expires_at >= Utc::now(),
        "만료되었거나 올바르지 않은 실행 요청",
    )?;
    Ok(ticket)
}

pub(super) fn claim(repository: &StorageRepository, id: &str) -> Result<()> {
    let mut claim = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(repository.run_path(id, "claim")?)
        .map_err(|_| "이미 시작한 실행 요청입니다.")?;
    write!(claim, "{}", std::process::id()).map_err(err)?;
    claim.sync_all().map_err(err)
}
