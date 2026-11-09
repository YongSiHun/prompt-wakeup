# 구조와 동작

## 의존성

React → 제한된 Tauri IPC → ScheduleService / SettingsService → SchedulerManager → ExecutionService → AI CLI 어댑터 / 터미널 어댑터 → 별도 실행 래퍼.

Zustand는 UI의 스냅샷, 탐지 결과, 오류만 관리합니다. 실제 예약 상태는 Rust의 Mutex로 보호됩니다. 스케줄러는 Tauri setup에서 Tokio 작업으로 시작하며 창 생명주기와 분리됩니다. 변경 이벤트를 구독한 UI가 Rust 스냅샷을 다시 조회합니다.

- ScheduleService: 예약 입력 검증, 생성·수정·취소·삭제 규칙.
- SchedulerManager: UTC 시각 감시, 중복 실행 억제, 복구, 저장 및 실행 서비스 호출.
- ExecutionService: 실행 요청 서명과 결과 파일 처리.
- AiCliAdapter: 탐지, 검증, 버전, LaunchPlan, 종료 추적 지원 여부. Codex / Claude / Gemini의 인자는 각각 분리했습니다.
- TerminalDetector / TerminalLauncher / TerminalAdapter: 설치 탐지와 실행 지원을 구분합니다. cmd, PowerShell, Windows Terminal + cmd, Git Bash, Terminal.app, iTerm2 구현이 있습니다. Warp는 탐지만 제공합니다.
- StorageRepository: schema_version=1 JSON, 동기화, 원자적 교체, 이전 파일 백업과 손상 복구.
- SettingsService / SecurityValidator / TrayManager: 설정 검증, 실행 경로·시간·프롬프트 검증, 트레이 및 명시적 종료.

## 실행 순서

1. 500ms 간격으로 UTC 예약 시각을 재평가합니다. 동일 시각의 모든 작업을 선택합니다.
2. launching 상태와 고유 ExecutionRecord를 저장합니다. 이 저장이 실패하면 실행하지 않습니다.
3. 사용자 데이터 폴더에 32바이트 키로 HMAC-SHA256 서명한 UUID 실행 요청을 작성합니다. 유효기간은 10분입니다.
4. 선택한 외부 터미널에서 현재 실행 파일의 --run-job UUID 모드를 실행합니다. 이 모드는 단일 인스턴스 플러그인 초기화 전에 처리됩니다.
5. 래퍼는 앱의 고정 사용자 데이터 경로에서만 요청을 읽습니다. UUID·서명·유효기간을 검증하고 create_new claim 파일로 재사용을 거부합니다.
6. CLI별 고정 인자를 구성하고 UTF-8 프롬프트를 stdin으로 전달한 뒤 EOF를 보냅니다. stdout/stderr는 외부 터미널에 상속합니다. 토큰·프롬프트를 실행 이력에 저장하지 않습니다.
7. CLI 종료 코드를 별도 결과 파일에 원자적으로 기록합니다. 메인 앱은 실제 종료 코드 0을 확인한 경우에만 completed로 바꿉니다.
8. 결과 없는 래퍼 종료는 unknown으로 기록합니다. 래퍼 프로세스가 살아 있거나 확인할 수 없으면 중복 실행을 보수적으로 막습니다. PID 재사용으로 결과 확인이 지연될 수 있습니다.

터미널 호스트의 spawn 성공은 CLI 작업 성공을 의미하지 않습니다. launched는 실행 요청이 전달된 상태입니다. CLI 실패는 자동 재시도하지 않으며 수동 실행마다 새로운 이력을 만듭니다.

## 시간과 복구

UTC와 IANA 시간대, 원래 로컬 시각을 함께 저장합니다. DST에서 존재하지 않거나 두 번 나타나는 시각은 거부하고 다른 시각 선택을 안내합니다. OS 시계와 경과 시간의 차이, 폴링 간격이 5초를 초과한 상황을 감지하여 절전/시각 변경으로 놓친 작업을 missed로 처리합니다. 정상 동작 중 5초 이내의 폴링 지연만 허용합니다.

재시작 시 지난 pending 작업은 missed이며 실행 중이던 작업은 자동 재실행하지 않습니다. 독립 래퍼의 결과 파일이 나중에 도착하면 반영합니다. 완전 종료는 신규 예약 감시를 멈추고 저장한 뒤 앱만 종료합니다. 이미 시작한 외부 CLI를 종료하는 코드는 없습니다. 실제 OS별 생존 여부는 수동 검증이 필요합니다.

JSON 세 파일 전체를 하나의 트랜잭션으로 저장하지는 않습니다. 실행 이력을 먼저 저장하고 실행 전 상태를 저장합니다. 재시작 시 미종료 실행 이력과 예약을 대조하여 재실행을 막습니다. 디스크 손상·강제 종료 상황에서 정확히 한 번 실행을 보장하지 않습니다.

## 저장과 권한

데이터 경로는 dirs::data_local_dir()/PromptWakeup입니다. Windows에서는 일반적으로 %LOCALAPPDATA%/PromptWakeup, macOS는 ~/Library/Application Support/PromptWakeup입니다.

schedules.json, settings.json, executions.json에는 schema_version과 data를 저장합니다. runs에는 실행 요청·claim·결과가 있으며 이력 보존 기간에 맞춰 정리합니다. 요청 파일은 결과 기록 후 삭제됩니다. runner.key는 사용자별 로컬 실행 요청 키이며 CLI 인증 자격 증명이 아닙니다.

Unix 디렉터리는 0700, Windows 디렉터리는 icacls로 상속을 제거하고 현재 사용자에게 접근을 부여합니다. 동기 쓰기는 Rust Mutex로 직렬화하고 래퍼는 별도 실행 결과 파일만 씁니다. 임시 파일을 같은 디렉터리에서 생성한 후 sync와 atomic persist를 수행합니다. Unix는 디렉터리도 sync합니다. 이전 JSON은 .bak 하나를 유지하며 손상 원본은 별도 이름으로 보존합니다. 미래 스키마는 자동 다운그레이드하지 않고 시작을 거부합니다. v1 이전 스키마가 없어 별도 이전 버전 마이그레이션은 없습니다.

HMAC은 다른 OS 사용자와 우발적 데이터 변조에 대한 방어입니다. 같은 사용자 권한을 가진 악성 프로그램을 격리하는 보안 경계는 아닙니다. 프롬프트는 암호화하지 않으며 백업에도 남을 수 있습니다. CLI 자체의 로그·인증 저장 정책은 해당 CLI가 관리합니다.

## 프로세스 및 UI 보안

프론트엔드에 범용 셸 실행 명령을 노출하지 않습니다. Tauri capability는 기본 이벤트와 명시적인 파일 선택만 허용하고 원격 콘텐츠를 로드하지 않습니다. 프로세스는 현재 사용자 환경에서 실행합니다. CLI 정책 확대, 관리자 상승, 보안 설정 변경을 하지 않습니다.

Windows batch shim은 고정 인자만 검증한 뒤 cmd의 raw_arg로 전달하고 프롬프트는 stdin으로 분리합니다. 배치 경로 및 cmd 기반 앱 경로의 셸 제어 문자는 거부합니다. PowerShell과 POSIX 셸은 신뢰한 래퍼 경로만 각 규칙으로 인용합니다. 프로세스 출력은 앱이 캡처·저장하지 않으므로 앱의 출력 로그 용량은 늘어나지 않습니다. 외부 터미널 스크롤백은 해당 터미널 설정을 따릅니다. 실행 메타데이터는 보존 기간으로 정리합니다.
