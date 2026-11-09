# PromptWakeup

## 개요

**Wake up your prompts on time.**

AI CLI 사용량 제한 해제 시각을 직접 입력해 프롬프트를 예약하는 Windows/macOS 데스크톱 앱입니다. 앱 실행 중 예약 시각에 실제 외부 터미널에서 Codex, Claude Code 또는 Gemini CLI를 비대화형으로 실행합니다. 창을 닫으면 시스템 트레이에서 예약을 감시합니다. 버전 1.0.0 · MIT 라이선스.

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
yarn lint
yarn typecheck
yarn test
# Windows 전용 전송 검사
yarn test:windows
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
