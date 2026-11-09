# 배포

현재 네이티브 빌드와 OS GUI 검증은 완료되지 않았습니다. 검증 범위는 [verification.md](verification.md)를 확인하세요.

## Windows

Windows 10/11 x64를 대상으로 yarn tauri build --no-bundle을 실행합니다. 산출물은 src-tauri/target/release/PromptWakeup.exe입니다. 설치 마법사를 만들지 않으며 정적 CRT 링크를 설정했습니다. 실제 Portable 이동 실행은 추가 검증이 필요합니다.

개발에는 Microsoft C++ Build Tools와 Rust stable이 필요합니다. 사용자 PC에는 [Microsoft WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)이 필요합니다. 런타임 설치·관리자 권한 상승·보안 예외 등록을 자동으로 수행하지 않습니다.

선택적 Authenticode 서명 Secrets:

- WINDOWS_CERTIFICATE: 개인키를 포함한 PFX의 Base64
- WINDOWS_CERTIFICATE_PASSWORD: PFX 비밀번호

서명·타임스탬프·검증 성공 시 signed, 인증서가 없으면 development-unsigned로 구분합니다. 서명으로 SmartScreen 평판 경고가 사라지는 것을 보장하지 않습니다.

## macOS

ARM64와 x64를 각각 빌드하고 .app을 ZIP으로 묶습니다. Hardened Runtime을 활성화하고 Apple Events 자동화 권한만 요청합니다. 터미널 실행 시 OS의 정식 Automation 승인을 사용합니다.

Developer ID Application 인증서와 Apple Developer 계정이 필요합니다. Secrets:

- APPLE_CERTIFICATE / APPLE_CERTIFICATE_PASSWORD: 개인키 포함 .p12의 Base64 및 비밀번호
- APPLE_SIGNING_IDENTITY: Developer ID Application 서명 ID
- APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID: 계정 이메일, 앱 전용 비밀번호, 팀 ID

development-unsigned, signed-unnotarized, signed-notarized를 파일명과 BUILD_MODE.txt로 구분합니다. 공증 완료 산출물은 stapler와 spctl 검증까지 통과해야 합니다. 부분 자격 증명 및 공증 실패는 빌드 실패로 처리합니다. 미서명 개발 빌드는 Gatekeeper 경고 또는 실행 차단 대상일 수 있습니다.

## CI와 저장

두 워크플로우는 workflow_dispatch 수동 실행만 허용합니다. push, PR, 태그 생성으로 자동 실행하지 않습니다. GitHub Actions에서 Build Windows 또는 Build macOS → Run workflow를 선택하세요. 린트·타입 검사·테스트·빌드·SHA-256·Artifact 업로드를 수행합니다.

Windows 빌드 성공 후 PromptWakeup-windows-x64-exe Artifact를 다운로드하고 압축을 풀면 PromptWakeup.exe가 나옵니다. 배포 안내·체크섬을 포함한 ZIP은 별도 package Artifact에 있습니다. macOS는 아키텍처별 ZIP Artifact를 제공합니다.

v\* 태그를 선택하고 publish_release를 켠 수동 실행에서만 Release를 게시합니다. 해당 게시 job에만 contents:write 권한을 부여합니다. 실제 Actions 실행 및 릴리스 게시는 아직 하지 않았습니다.

최초 네트워크 허용 환경에서 yarn install과 Cargo로 실제 잠금 파일을 생성하고 커밋한 뒤 --frozen-lockfile/--locked 설치로 고정해야 합니다. 해석되지 않은 임시 yarn.lock은 삭제했습니다.

데이터는 실행 파일 옆이 아닌 사용자 데이터 경로의 PromptWakeup 폴더에 저장합니다. Windows는 보통 %LOCALAPPDATA%, macOS는 ~/Library/Application Support 아래입니다. 프롬프트는 로컬 JSON과 백업에 평문으로 남을 수 있습니다. 실행 이력에는 프롬프트·환경 변수·콘솔 원문을 저장하지 않습니다.
