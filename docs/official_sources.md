# 확인한 공식 자료

2026-11-09 조회. 아래 자료의 실행 형태를 확인했으며 로컬에 실제 CLI를 설치·인증하거나 실행하지는 않았습니다.

- [OpenAI 공식 Codex 비대화형 실행](https://developers.openai.com/codex/noninteractive): codex exec -, UTF-8 stdin과 진행 출력. 링크는 현재 ChatGPT Learn 문서로 이동합니다.
- [Claude Code 프로그래밍 실행](https://code.claude.com/docs/en/headless): claude -p와 종료 코드. 권한 확대 옵션을 자동으로 추가하지 않았습니다.
- [Gemini CLI Headless](https://geminicli.com/docs/cli/headless/), [stdin 파이프 자동화](https://geminicli.com/docs/cli/tutorials/automation/): gemini -p의 고정 작업 지시와 stdin 컨텍스트.
- [Tauri 2 트레이](https://v2.tauri.app/learn/system-tray/), [Single Instance](https://v2.tauri.app/plugin/single-instance/): 네이티브 트레이 및 중복 메인 앱 방지.
- [Tauri 2 빌드 선행 조건](https://v2.tauri.app/start/prerequisites/), [Windows 배포](https://v2.tauri.app/distribute/windows-installer/): MSVC와 WebView2 의존성.
- [Tauri macOS 서명](https://v2.tauri.app/distribute/sign/macos/), [환경 변수](https://v2.tauri.app/reference/environment-variables/): Developer ID, 인증서·공증 Secrets 구성.
- [iTerm2 AppleScript](https://iterm2.com/documentation-scripting.html): 새 세션 명령 실행. 이 API의 지원이 곧 이 앱의 실기 검증 완료를 의미하지 않습니다.
- [Warp Launch Configurations](https://docs.warp.dev/terminal/sessions/launch-configurations/): 기존 방식의 한계를 고려해 현재 실행 지원을 활성화하지 않았습니다.
- [Git for Windows wrapper](https://gitforwindows.org/git-wrapper.html): 터미널 런처와 실제 셸의 구분.
- [GitHub 호스티드 러너](https://docs.github.com/en/actions/reference/runners/github-hosted-runners): macOS ARM64/Intel 러너 레이블.
- [Tauri notification Rust 구현](https://docs.rs/tauri-plugin-notification/latest/src/tauri_plugin_notification/desktop.rs.html): 데스크톱 permission_state/request_permission 응답의 한계.

- [Tauri 설정 참조](https://v2.tauri.app/reference/config/): Hardened Runtime과 Info.plist 병합 설정.
