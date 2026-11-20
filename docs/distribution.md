# 배포 안내

## 수동 빌드

GitHub Actions에서 **1. Build Windows** 또는 **2. Build macOS → Run workflow**를 선택합니다. push·PR·태그 생성으로는 실행되지 않습니다.

| 플랫폼            | 빌드 성공 후 다운로드                                             |
| ----------------- | ----------------------------------------------------------------- |
| Windows x64       | `PromptWakeup-windows-x64-exe` Artifact를 풀면 `PromptWakeup.exe` |
| macOS ARM64 / x64 | 아키텍처별 Artifact에 `PromptWakeup.app`만 포함한 ZIP             |

빌드 결과는 GitHub Actions의 Artifact에서 다운로드합니다. GitHub Release는 직접 만들고 필요한 파일을 첨부합니다. 워크플로는 읽기 권한만 사용합니다.

## 실행 환경과 데이터

- Windows 10/11 x64: WebView2 Evergreen Runtime이 필요합니다. 설치 마법사 없이 EXE를 배포합니다.
- macOS: 외부 터미널 자동화 없이 앱 내 작업 프로세스로 실행합니다.
- 예약과 설정은 OS 사용자 데이터 폴더의 PromptWakeup 디렉터리에 저장됩니다. 프롬프트는 JSON과 백업에 평문으로 남을 수 있습니다. 실행 당시 프롬프트와 메타데이터는 `executions.json`, 터미널 stdout·stderr는 `runs/<실행 ID>.output` UTF-8 텍스트 파일에 저장됩니다. Windows 기본 위치는 `%LOCALAPPDATA%\PromptWakeup`, macOS는 `~/Library/Application Support/PromptWakeup`입니다. 실행이력 화면은 최근 출력 256KiB를 표시합니다. 전체 원문은 카드의 파일·폴더 열기로 확인할 수 있습니다.

현재 빌드·실행 검증 범위는 [검증 현황](verification.md)을 확인하세요.

## CLI 탐색과 macOS 권한

- CLI 경로는 사용자가 저장한 경로를 우선합니다. 자동 탐색은 PATH와 일반 설치 경로를 사용합니다. Homebrew 경로는 Apple Silicon과 Intel 모두 후보에 포함하며 필수 설치 위치가 아닙니다.
- macOS에서는 로그인 셸의 PATH를 읽어 Finder 실행 시에도 버전 검사와 실제 실행에 적용합니다. 셸 조회는 5초로 제한하고, 실패하면 앱이 받은 PATH와 기본 탐색 경로를 사용합니다. 수동 지정한 실행 파일의 폴더도 자식 프로세스 PATH 앞에 추가합니다. PATH는 프로세스별로 캐시하므로 셸 설정을 바꾸면 앱을 다시 시작하세요.
- 명령 인자는 CLI별 기본값으로 코드에 정의되어 있습니다: Codex는 `exec -`, Claude는 `-p`, Gemini는 `-p "Execute the task supplied on stdin"`입니다. 프롬프트 원문은 stdin으로 전달합니다. 승인·샌드박스 우회 인자는 추가하지 않으며 각 CLI의 기존 인증·권한 설정을 따릅니다.
- macOS 데이터 폴더는 사용자 전용 권한(`0700`)으로 제한하고, 선택한 CLI의 실행 비트를 검사합니다. macOS 개인정보 보호 권한이나 CLI 인증을 앱이 자동으로 허용하지는 않습니다.
- macOS 빌드 워크플로는 ARM64와 x64에서 검사·테스트·패키징을 수행합니다. 최종 확인은 앱을 Finder에서 열어 CLI 탐색·버전 확인·예약 실행을 수행해야 합니다. 터미널의 PATH가 있는 실행만으로는 검증이 충분하지 않습니다.
