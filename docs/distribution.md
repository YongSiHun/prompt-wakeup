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
- 예약과 설정은 OS 사용자 데이터 폴더의 PromptWakeup 디렉터리에 저장됩니다. 프롬프트는 JSON과 백업에 평문으로 남을 수 있습니다. 실행 이력의 메타데이터는 `executions.json`, 터미널 stdout·stderr는 `runs/<실행 ID>.output` UTF-8 텍스트 파일에 저장됩니다. Windows 기본 위치는 `%LOCALAPPDATA%\PromptWakeup`, macOS는 `~/Library/Application Support/PromptWakeup`입니다. 실행 결과 카드의 전체 경로와 파일·폴더 열기로 확인할 수 있습니다.

현재 빌드·실행 검증 범위는 [검증 현황](verification.md)을 확인하세요.
