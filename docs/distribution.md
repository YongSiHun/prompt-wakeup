# 배포 안내

## 수동 빌드

GitHub Actions에서 **Build Windows** 또는 **Build macOS → Run workflow**를 선택합니다. push·PR·태그 생성으로는 실행되지 않습니다.

| 플랫폼              | 빌드 성공 후 다운로드                                              |
| ------------------- | ------------------------------------------------------------------ |
| Windows x64         | `PromptWakeup-windows-x64-exe` Artifact를 풀면 `PromptWakeup.exe`  |
| Windows 배포 패키지 | `PromptWakeup-windows-x64-package` Artifact에 안내·체크섬 포함 ZIP |
| macOS ARM64 / x64   | 아키텍처별 Artifact에 `PromptWakeup.app` 포함 ZIP·체크섬           |

`v*` 태그를 선택하고 `publish_release`를 켠 수동 실행에서만 GitHub Release에 게시합니다. 게시 job에만 쓰기 권한을 부여합니다.

## 실행 환경과 데이터

- Windows 10/11 x64: WebView2 Evergreen Runtime이 필요합니다. 설치 마법사 없이 EXE를 배포합니다.
- macOS: 외부 터미널 자동화 없이 앱 내 작업 프로세스로 실행합니다.
- 예약과 설정은 OS 사용자 데이터 폴더의 PromptWakeup 디렉터리에 저장됩니다. 프롬프트는 JSON과 백업에 평문으로 남을 수 있습니다. 실행 이력의 메타데이터는 `executions.json`, 터미널 stdout·stderr는 `runs/<실행 ID>.output` UTF-8 텍스트 파일에 저장됩니다. Windows 기본 위치는 `%LOCALAPPDATA%\PromptWakeup`, macOS는 `~/Library/Application Support/PromptWakeup`입니다. 실행 결과 카드의 전체 경로와 파일·폴더 열기로 확인할 수 있습니다.

## 서명 및 공증

서명 빌드에는 GitHub Secrets에 아래 값을 등록합니다. Windows 수동 빌드의 기본값은 `signed_build=false`, `publish_release=false`이며, 인증서 없이 개발 빌드를 만들 수 있습니다. 서명이 필요한 경우 `signed_build=true`를 선택하세요. Release 게시에는 항상 서명이 필요하며, 서명이 요구되는 빌드에서 인증서나 비밀번호가 없으면 빌드를 중단합니다. macOS는 인증서가 없으면 미서명 개발 빌드로 구분합니다.

| 플랫폼     | Secrets                                                                                                                            |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Windows    | `WINDOWS_CERTIFICATE`: 개인키 포함 PFX의 Base64, `WINDOWS_CERTIFICATE_PASSWORD`: 비밀번호                                          |
| macOS 서명 | `APPLE_CERTIFICATE`: 개인키 포함 P12의 Base64, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`: Developer ID Application ID |
| macOS 공증 | `APPLE_ID`, `APPLE_PASSWORD`: 앱 전용 비밀번호, `APPLE_TEAM_ID`                                                                    |

Windows는 `signed` / `development-unsigned`, macOS는 `signed-notarized` / `signed-unnotarized` / `development-unsigned`를 파일명과 `BUILD_MODE.txt`로 표시합니다. 서명·공증 과정의 실패는 빌드를 중단합니다. 미서명 빌드는 SmartScreen 또는 Gatekeeper 경고·차단 대상이 될 수 있습니다.

Windows 서명 후 `signtool verify /pa /all /tw` 및 Authenticode 신뢰·타임스탬프를 검증합니다. 서명은 게시자를 확인하는 수단이며, 신규 파일의 SmartScreen 평판 경고가 즉시 사라지는 것을 보장하지 않습니다. [Microsoft SmartScreen 안내](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation)를 참고하세요.

현재 빌드·실행 검증 범위는 [검증 현황](verification.md)을 확인하세요.
