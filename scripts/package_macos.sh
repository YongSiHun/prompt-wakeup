#!/usr/bin/env bash
set -euo pipefail

yarn tauri build --target "$BUILD_TARGET" --bundles app
app="src-tauri/target/$BUILD_TARGET/release/bundle/macos/PromptWakeup.app"
name="PromptWakeup-macos-$BUILD_ARCH"
mkdir -p artifacts
ditto -c -k --sequesterRsrc --keepParent "$app" "artifacts/$name.zip"
