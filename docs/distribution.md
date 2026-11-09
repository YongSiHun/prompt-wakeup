# 배포와 서명

현재 저장소는 네이티브 빌드 및 실기 GUI 검증을 통과하지 않았습니다. 배포 준비 완료로 간주하지 마세요. 상세 결과는 validation.md에 기록합니다.

## Windows

Microsoft C++ Build Tools와 Rust stable MSVC 도구 체인으로 빌드합니다. yarn tauri build --no-bundle 결과의 src-tauri/target/release/PromptWakeup.exe를 설치 마법사 없이 배포하도록 구성했습니다. Windows 10/11 x64가 대상입니다. MSVC 정적 CRT 링크를 지정했습니다. 실제 단일 EXE 이동 실행은 아직 검증하지 못했습니다.

Microsoft Edge WebView2 Evergreen Runtime이 필요합니다. 미설치 PC에서는 [Microsoft 공식 배포 페이지](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)의 런타임을 사용자 또는 조직의 정식 절차로 설치해야 합니다. 앱이 런타임 설치, UAC 상승 또는 Defender 예외를 자동 설정하지 않습니다. 설치 폴더가 아닌 사용자 데이터 폴더에 예약을 저장합니다.

CI는 선택적인 Authenticode 서명을 지원합니다. GitHub Secrets WINDOWS_CERTIFICATE는 개인키를 포함한 PFX의 Base64, WINDOWS_CERTIFICATE_PASSWORD는 그 비밀번호입니다. 인증서는 Windows 코드 서명 용도로 유효해야 합니다. 서명 도구가 확인·타임스탬프·검증을 완료한 경우만 signed로 패키징합니다. Secrets가 없으면 development-unsigned입니다. 서명만으로 SmartScreen 평판 경고가 사라지는 것을 보장하지 않습니다.

## macOS

ARM64는 macos-15, x64는 macos-15-intel 호스티드 러너에서 개별 빌드합니다. 각각 .app을 ZIP으로 배포합니다. 최소 macOS 12.0 설정이며 해당 버전에서 실제 호환성 검증은 아직 하지 못했습니다.

외부 배포용 Developer ID Application 인증서 및 Apple Developer Program 가입이 필요합니다. GitHub Secrets:

- APPLE_CERTIFICATE: 개인키 포함 .p12의 Base64
- APPLE_CERTIFICATE_PASSWORD: .p12 비밀번호
- APPLE_SIGNING_IDENTITY: Developer ID Application 서명 ID
- APPLE_ID: Apple Developer 계정 이메일
- APPLE_PASSWORD: 앱 전용 비밀번호
- APPLE_TEAM_ID: Developer Team ID

Tauri의 서명·Hardened Runtime·공증 흐름을 사용합니다. Entitlements.plist는 Apple Events 자동화만 요청합니다. Info.plist에는 터미널 자동화 목적을 한국어로 표시합니다. 터미널 자동화 권한은 실제 사용자 OS 승인으로 처리하며 거절 시 안내하고 우회하지 않습니다.

서명 정보가 없으면 development-unsigned, 서명만 있으면 signed-unnotarized, 서명·공증 정보가 모두 있고 공증 및 stapler/spctl 검증을 통과하면 signed-notarized입니다. 부분적으로 제공한 자격 증명이나 공증 실패는 CI 실패로 처리합니다. 서명·공증하지 않은 개발용 빌드는 Gatekeeper 경고 또는 실행 차단 대상이 될 수 있습니다. 보안 경고 우회 명령을 제공하지 않습니다.

## 릴리스 전

네트워크 가능한 개발 환경에서 yarn install과 cargo check로 실제 yarn.lock/Cargo.lock을 생성하고 검토·커밋하세요. 현재 yarn.lock은 해석되지 않은 부트스트랩 파일입니다. 재현 가능한 릴리스로 전환할 때 CI를 yarn install --frozen-lockfile 및 cargo --locked로 변경하세요.

두 워크플로우는 타입 검사, UI 테스트, Rust 검사·테스트, release 빌드, SHA-256, Artifact 업로드를 수행하도록 작성했습니다. 태그 v\*의 별도 release job만 contents:write 권한을 가지며 성공한 빌드 ZIP과 체크섬을 게시합니다. GitHub에 푸시하거나 Actions를 실행하지 않았습니다. 서명 인증서를 소스에 넣지 마세요.

압축 파일명과 BUILD_MODE.txt로 개발용/서명/공증 상태를 구분합니다. 최종 수동 검증과 설치 의존성 검증을 마치기 전에는 일반 사용자용 안정 릴리스로 홍보하지 마세요.
