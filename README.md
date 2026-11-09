# PromptWakeup

## 개요

**Wake up your prompts on time.**

AI CLI 사용량 제한 해제 시각을 직접 입력해 프롬프트를 예약하는 Windows/macOS 데스크톱 앱입니다. 앱 실행 중 예약 시각에 실제 외부 터미널에서 Codex, Claude Code 또는 Gemini CLI를 비대화형으로 실행합니다. 창을 닫으면 시스템 트레이에서 예약을 감시합니다. 버전 1.0.0 · MIT 라이선스.

Prettier는 TypeScript·JavaScript·CSS·JSON·Markdown·YAML을 정리하고, Rust는 `cargo fmt`로 정리합니다. CI에서도 두 포맷 검사와 lint를 실행합니다.

## 실행 파일 선택

예약 생성·수정 화면에서 AI CLI 종류를 고른 뒤 **AI CLI 실행 파일 선택**으로 해당 실행 파일을 지정하고, **터미널 실행 파일 선택**으로 지원되는 터미널을 지정할 수 있습니다. 선택한 경로는 검사 후 저장되어 다음 예약에서도 사용할 수 있습니다. Windows의 CLI는 `.exe`, `.cmd`, `.bat`을 지원하며, macOS 터미널은 지원되는 `.app`을 선택하세요.

설정 화면에서도 실행 파일을 지정하고 기본 AI CLI와 터미널을 저장할 수 있습니다. 같은 종류의 터미널이 여러 경로에 설치되어 있어도 각각 구분됩니다. 기존 예약의 실행 경로를 바꾸려면 예약을 수정한 뒤 저장하세요.

## 기술 스택

- React, TypeScript strict, Vite, Tailwind CSS, Zustand
- Tauri 2, Rust, Tokio, Serde, 로컬 JSON
- Yarn, Cargo, GitHub Actions

## 명령어

개발 환경: Node.js LTS, Yarn 1.22.22, Rust stable. Windows는 Microsoft C++ Build Tools와 WebView2, macOS는 Xcode Command Line Tools가 필요합니다.

```sh
npm install --global yarn@1.22.22
yarn install
yarn tauri dev
```

```sh
yarn format
yarn format:check
yarn lint
yarn lint:fix
yarn typecheck
yarn test
# Windows 전용 전송 검사
yarn test:windows
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D clippy::cognitive_complexity
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml
```

```sh
# Windows Portable EXE
yarn tauri build --no-bundle
# src-tauri/target/release/PromptWakeup.exe

# macOS APP
yarn tauri build --bundles app
# src-tauri/target/release/bundle/macos/PromptWakeup.app
```
