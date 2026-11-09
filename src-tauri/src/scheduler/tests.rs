use crate::security::SecurityValidator;
use super::*;use crate::models::*;use crate::ai_cli::AiCliAdapter;
fn setup()->(tempfile::TempDir,Arc<SchedulerManager>){let d=tempfile::tempdir().unwrap();let m=SchedulerManager::open(StorageRepository::at(d.path().join("data")).unwrap()).unwrap();(d,m)}
pub(crate) fn input(dir:&std::path::Path)->ScheduleInput{let cli=dir.join(if cfg!(windows){"codex.cmd"}else{"codex"});std::fs::write(&cli,if cfg!(windows){"@echo off\r\necho fake-cli 1.0\r\nexit /b 0\r\n"}else{"#!/bin/sh\necho fake-cli 1.0\n"}).unwrap();#[cfg(unix)]{use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&cli,std::fs::Permissions::from_mode(0o700)).unwrap();}let terminal=crate::terminal::detection::TerminalDetector::detect().into_iter().find(|t|t.supported).unwrap();let time=Utc::now()+chrono::Duration::hours(1);let time=DateTime::from_timestamp(time.timestamp(),0).unwrap();ScheduleInput{name:"한글 예약".into(),cli:CliKind::Codex,cli_path:cli.to_string_lossy().into(),terminal,working_directory:dir.to_string_lossy().into(),prompt:"한글 ' \" & $(touch bad) \n다음 줄 %PATH%".into(),scheduled_at:time,timezone:"Asia/Seoul".into(),local_time:time.with_timezone(&chrono_tz::Asia::Seoul).format("%Y-%m-%dT%H:%M:%S").to_string(),keep_open:false}}
#[test]fn create_edit_cancel_delete_preserves_history(){let (d,m)=setup();m.upsert(None,input(d.path())).unwrap();let first=m.snapshot().unwrap().schedules[0].clone();let mut v=first.input.clone();v.name="수정".into();m.upsert(Some(first.id.clone()),v).unwrap();assert_eq!(m.snapshot().unwrap().schedules[0].input.name,"수정");m.action(first.id.clone(),"cancel").unwrap();assert_eq!(m.snapshot().unwrap().schedules[0].status,ScheduleStatus::Canceled);m.action(first.id,"delete").unwrap();assert!(m.snapshot().unwrap().schedules.is_empty());}
#[test]fn simultaneous_schedules_remain_independent(){let (d,m)=setup();let i=input(d.path());m.upsert(None,i.clone()).unwrap();m.upsert(None,i).unwrap();let s=m.snapshot().unwrap();assert_eq!(s.schedules.len(),2);assert_ne!(s.schedules[0].id,s.schedules[1].id);}
#[test]fn duplicate_running_job_is_rejected(){let (d,m)=setup();m.upsert(None,input(d.path())).unwrap();let id={let mut s=m.state.lock().unwrap();s.schedules[0].status=ScheduleStatus::Launched;s.schedules[0].id.clone()};assert!(m.launch(&id,true).is_err());assert!(m.snapshot().unwrap().executions.is_empty());}
#[test]fn restart_restores_future_and_marks_past_missed(){let (d,m)=setup();m.upsert(None,input(d.path())).unwrap();let mut s=m.snapshot().unwrap();s.schedules.push(s.schedules[0].clone());s.schedules[1].id=uuid::Uuid::new_v4().to_string();s.schedules[1].input.scheduled_at=Utc::now()-chrono::Duration::seconds(10);m.save(&s).unwrap();let reopened=SchedulerManager::open(m.repository.clone()).unwrap();assert_eq!(reopened.snapshot().unwrap().schedules[0].status,ScheduleStatus::Pending);assert_eq!(reopened.snapshot().unwrap().schedules[1].status,ScheduleStatus::Missed);}
#[test]fn resume_gap_marks_due_missed_without_launch(){let(d,m)=setup();m.upsert(None,input(d.path())).unwrap();let then=m.snapshot().unwrap().schedules[0].input.scheduled_at;m.tick(then,true).unwrap();assert_eq!(m.snapshot().unwrap().schedules[0].status,ScheduleStatus::Missed);assert!(m.snapshot().unwrap().executions.is_empty());}
#[test]fn utc_local_roundtrip_and_dst_rejection(){let(d,_)=setup();let mut v=input(d.path());assert!(SecurityValidator::input(&v,true).is_ok());v.timezone="America/New_York".into();v.local_time="2027-11-07T01:30:00".into();v.scheduled_at="2027-11-07T05:30:00Z".parse().unwrap();assert!(SecurityValidator::input(&v,false).unwrap_err().contains("DST"));v.local_time="2027-03-14T02:30:00".into();assert!(SecurityValidator::input(&v,false).is_err());}
#[test]fn invalid_path_and_empty_prompt_rejected(){let(d,_)=setup();let mut v=input(d.path());v.prompt=" ".into();assert!(SecurityValidator::input(&v,true).is_err());v.prompt="ok".into();v.cli_path="relative/codex".into();assert!(SecurityValidator::input(&v,true).is_err());}
#[test]fn adapters_keep_prompts_out_of_arguments(){let(d,_)=setup();let v=input(d.path());let plans=[crate::ai_cli::adapters::CodexAdapter.build_launch_plan(&v),crate::ai_cli::adapters::ClaudeAdapter.build_launch_plan(&v),crate::ai_cli::adapters::GeminiAdapter.build_launch_plan(&v)];assert_eq!(plans[0].arguments,vec!["exec","-"]);assert_eq!(plans[1].arguments,vec!["-p"]);assert_eq!(plans[2].arguments,vec!["-p","Execute the task supplied on stdin"]);for p in plans{assert!(!p.arguments.iter().any(|a|a==&v.prompt));assert!(p.environment.is_empty());assert_eq!(p.prompt_delivery,"stdin_utf8");}}
#[test]fn signed_ticket_tampering_detected(){use hmac::{Hmac,Mac};use sha2::Sha256;let(d,m)=setup();let s=Schedule{id:uuid::Uuid::new_v4().to_string(),input:input(d.path()),status:ScheduleStatus::Pending};let id=uuid::Uuid::new_v4().to_string();ExecutionService::prepare(&m.repository,&s,&id).unwrap();let p=m.repository.run_path(&id,"ticket").unwrap();let mut ticket:serde_json::Value=serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();let mut mac=Hmac::<Sha256>::new_from_slice(&m.repository.key(false).unwrap()).unwrap();let payload=ticket["payload"].as_str().unwrap().to_string();mac.update(payload.as_bytes());assert!(mac.verify_slice(&hex::decode(ticket["signature"].as_str().unwrap()).unwrap()).is_ok());ticket["payload"]=serde_json::Value::String(format!("{payload}x"));let mut mac=Hmac::<Sha256>::new_from_slice(&m.repository.key(false).unwrap()).unwrap();mac.update(ticket["payload"].as_str().unwrap().as_bytes());assert!(mac.verify_slice(&hex::decode(ticket["signature"].as_str().unwrap()).unwrap()).is_err());}
#[test]fn result_sets_completion_only_after_cli_zero(){let(d,m)=setup();m.upsert(None,input(d.path())).unwrap();let schedule=m.snapshot().unwrap().schedules[0].clone();let id=uuid::Uuid::new_v4().to_string();{let mut s=m.state.lock().unwrap();s.schedules[0].status=ScheduleStatus::Launched;s.executions.push(ExecutionRecord{id:id.clone(),schedule_id:schedule.id,name:"test".into(),cli:CliKind::Codex,started_at:Utc::now(),ended_at:None,status:ExecutionStatus::Launched,exit_code:None,error:None});}crate::storage::atomic(&m.repository.run_path(&id,"result").unwrap(),&RunResult{execution_id:id,ended_at:Utc::now(),exit_code:Some(0),error:None}).unwrap();m.tick(Utc::now(),false).unwrap();assert_eq!(m.snapshot().unwrap().schedules[0].status,ScheduleStatus::Completed);let id=m.snapshot().unwrap().schedules[0].id.clone();m.action(id,"delete").unwrap();assert_eq!(m.snapshot().unwrap().executions.len(),1);}
#[test]fn corrupt_json_recovers_backup(){let(_,m)=setup();m.repository.save("test.json",&vec![1]).unwrap();m.repository.save("test.json",&vec![2]).unwrap();std::fs::write(m.repository.root.join("test.json"),"{bad").unwrap();let mut warnings=vec![];let data:Vec<i32>=m.repository.load("test.json",&mut warnings).unwrap();assert_eq!(data,vec![1]);assert_eq!(warnings.len(),1);}
#[test]fn path_traversal_run_id_rejected(){let(_,m)=setup();assert!(m.repository.run_path("../../settings","ticket").is_err());}
#[test]fn shell_quotes_do_not_expose_prompt_as_code(){assert_eq!(crate::security::ps_quote("a'b"),"'a''b'");assert_eq!(crate::security::sh_quote("a'b"),"'a'\"'\"'b'");}
#[test]fn version_detects_stub_cli(){let(d,_)=setup();let v=input(d.path());assert!(crate::ai_cli::version(&v.cli_path).unwrap().contains("fake-cli"));}

#[test]
fn simultaneous_due_jobs_launch_once_each() {
    let (dir, mut manager) = setup();
    let launches = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = launches.clone();
    Arc::get_mut(&mut manager).unwrap().terminal_launcher = Arc::new(move |_, _| {count.fetch_add(1, Ordering::SeqCst);Ok(())});
    let first = input(dir.path());
    manager.upsert(None, first.clone()).unwrap();
    manager.upsert(None, first).unwrap();
    let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
    {
        let mut state = manager.state.lock().unwrap();
        for schedule in &mut state.schedules {
            schedule.input.scheduled_at = now;
            schedule.input.local_time = now.with_timezone(&chrono_tz::Asia::Seoul).format("%Y-%m-%dT%H:%M:%S").to_string();
        }
    }
    manager.tick(Utc::now(), false).unwrap();
    manager.tick(Utc::now(), false).unwrap();
    assert_eq!(launches.load(Ordering::SeqCst), 2);
    assert_eq!(manager.snapshot().unwrap().executions.len(), 2);
}
#[test]
fn changed_future_time_cannot_launch_from_stale_due_list() {
    let (dir, manager) = setup();
    manager.upsert(None, input(dir.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    assert!(manager.launch(&id, false).is_err());
    assert!(manager.snapshot().unwrap().executions.is_empty());
}
#[test]
fn persistence_failure_does_not_automatically_retry_launch() {
    let (dir, manager) = setup();
    manager.upsert(None, input(dir.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    let path = manager.repository.root.join("settings.json");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(manager.launch(&id, true).is_err());
    let state = manager.snapshot().unwrap();
    assert_eq!(state.schedules[0].status, ScheduleStatus::Failed);
    assert_eq!(state.executions.len(), 1);
    assert_eq!(state.executions[0].status, ExecutionStatus::Failed);
    manager.tick(Utc::now(), false).unwrap();
    assert_eq!(manager.snapshot().unwrap().executions.len(), 1);
}
