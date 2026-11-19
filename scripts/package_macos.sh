#!/usr/bin/env bash
set -euo pipefail

package_bundle() {
  mkdir -p "artifacts/$name"
  cp docs/distribution.md "artifacts/$name/distribution.md"
  ditto "$app" "artifacts/$name/PromptWakeup.app"
  (cd artifacts && ditto -c -k --sequesterRsrc --keepParent "$name" "$name.zip")
  (cd artifacts && shasum -a 256 "$name.zip" > "$name.zip.sha256")
}

yarn tauri build --target "$BUILD_TARGET" --bundles app
app="src-tauri/target/$BUILD_TARGET/release/bundle/macos/PromptWakeup.app"
name="PromptWakeup-macos-$BUILD_ARCH"
package_bundle
