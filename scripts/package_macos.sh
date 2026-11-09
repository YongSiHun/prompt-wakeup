#!/usr/bin/env bash
set -euo pipefail
mode=development-unsigned
if [[ -n "$APPLE_CERTIFICATE" || -n "$APPLE_SIGNING_IDENTITY" ]]; then
  test -n "$APPLE_CERTIFICATE" && test -n "$APPLE_CERTIFICATE_PASSWORD" && test -n "$APPLE_SIGNING_IDENTITY"
  mode=signed-unnotarized
  if [[ -n "$APPLE_ID" || -n "$APPLE_PASSWORD" || -n "$APPLE_TEAM_ID" ]]; then
    test -n "$APPLE_ID" && test -n "$APPLE_PASSWORD" && test -n "$APPLE_TEAM_ID"
    mode=signed-notarized
  else
    unset APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
  fi
else
  unset APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
fi
yarn tauri build --target "$BUILD_TARGET" --bundles app
app="src-tauri/target/$BUILD_TARGET/release/bundle/macos/PromptWakeup.app"
if [[ "$mode" != development-unsigned ]]; then codesign --verify --deep --strict "$app"; fi
if [[ "$mode" == signed-notarized ]]; then
  xcrun stapler validate "$app"
  spctl --assess --type execute --verbose "$app"
fi
mkdir -p artifacts
name="PromptWakeup-macos-$BUILD_ARCH-$mode"
mkdir -p "artifacts/$name"
cp docs/distribution.md "artifacts/$name/distribution.md"
printf '%s\n' "$mode" > "artifacts/$name/BUILD_MODE.txt"
ditto "$app" "artifacts/$name/PromptWakeup.app"
(cd artifacts && ditto -c -k --sequesterRsrc --keepParent "$name" "$name.zip")
(cd artifacts && shasum -a 256 "$name.zip" > "$name.zip.sha256")
