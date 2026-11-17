# PromptWakeup

AI CLI 예약 실행 앱 · Win/macOS · MIT

- Codex / Claude Code / Gemini CLI
- 예약: 작업 폴더·프롬프트·실행 시각 지정
- 실행: 내장 터미널·출력 저장·이력 삭제
- 닫기: 트레이 유지 / 종료
- CLI 설정: 이름 선택·경로 지정·재탐색

## Stack

- UI: React 19 · TS · Vite 6 · Tailwind CSS 4 · Zustand 5
- Desktop: Tauri 2 · Rust · Tokio
- Data: 로컬 JSON · Serde · Chrono
- Test: Vitest · Testing Library · Cargo test
- Quality/CI: ESLint · SonarJS · Prettier · rustfmt · Clippy · GitHub Actions

## Dev

Node LTS · Yarn 1.22 · Rust stable. Win: MSVC Build Tools + WebView2. macOS: Xcode CLI Tools.

```sh
npm i -g yarn@1.22.22
yarn install
yarn tauri dev
```

## Check

```sh
yarn format:check
yarn lint
yarn typecheck
yarn test
yarn test:windows
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D clippy::cognitive_complexity
cargo test --manifest-path src-tauri/Cargo.toml
```

## Build

```sh
yarn tauri build --no-bundle    # Win EXE
yarn tauri build --bundles app # macOS APP
```

수동 CI·서명·저장 경로: [배포](docs/distribution.md) · [검증](docs/verification.md)
