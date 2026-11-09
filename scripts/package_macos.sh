#!/usr/bin/env bash
set -euo pipefail

configure_notarization() {
  if [[ -z "$APPLE_ID" && -z "$APPLE_PASSWORD" && -z "$APPLE_TEAM_ID" ]]; then
    unset APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
    return
  fi
  test -n "$APPLE_ID" && test -n "$APPLE_PASSWORD" && test -n "$APPLE_TEAM_ID"
  mode=signed-notarized
}
configure_signing() {
  mode=development-unsigned
  if [[ -z "$APPLE_CERTIFICATE" && -z "$APPLE_SIGNING_IDENTITY" ]]; then
    unset APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
    return
  fi
  test -n "$APPLE_CERTIFICATE" && test -n "$APPLE_CERTIFICATE_PASSWORD" && test -n "$APPLE_SIGNING_IDENTITY"
  mode=signed-unnotarized
  configure_notarization
}
verify_bundle() {
  if [[ "$mode" == development-unsigned ]]; then return; fi
  codesign --verify --deep --strict "$app"
  if [[ "$mode" == signed-notarized ]]; then
    xcrun stapler validate "$app"
    spctl --assess --type execute --verbose "$app"
  fi
}
package_bundle() {
  mkdir -p "artifacts/$name"
  cp docs/distribution.md "artifacts/$name/distribution.md"
  printf '%s\n' "$mode" > "artifacts/$name/BUILD_MODE.txt"
  ditto "$app" "artifacts/$name/PromptWakeup.app"
  (cd artifacts && ditto -c -k --sequesterRsrc --keepParent "$name" "$name.zip")
  (cd artifacts && shasum -a 256 "$name.zip" > "$name.zip.sha256")
}

configure_signing
yarn tauri build --target "$BUILD_TARGET" --bundles app
app="src-tauri/target/$BUILD_TARGET/release/bundle/macos/PromptWakeup.app"
verify_bundle
name="PromptWakeup-macos-$BUILD_ARCH-$mode"
package_bundle
