$ErrorActionPreference = 'Stop'
$mode = 'development-unsigned'
if ($env:CERTIFICATE) {
  $pfxPath = Join-Path $env:RUNNER_TEMP 'prompt-wakeup.pfx'
  try {
    [IO.File]::WriteAllBytes($pfxPath, [Convert]::FromBase64String($env:CERTIFICATE))
    $cert = Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString $env:CERTIFICATE_PASSWORD -AsPlainText -Force)
    $tool = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe' | Sort-Object FullName -Descending | Select-Object -First 1
    if (!$tool) { throw 'signtool not found' }
    & $tool.FullName sign /sha1 $cert.Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 src-tauri/target/release/PromptWakeup.exe
    if ($LASTEXITCODE -ne 0) { throw 'Signing failed' }
    & $tool.FullName verify /pa src-tauri/target/release/PromptWakeup.exe
    if ($LASTEXITCODE -ne 0) { throw 'Signature verification failed' }
    $mode = 'signed'
  } finally {
    Remove-Item -LiteralPath $pfxPath -Force -ErrorAction SilentlyContinue
    if ($cert) { Remove-Item -LiteralPath ('Cert:\CurrentUser\My\' + $cert.Thumbprint) -ErrorAction SilentlyContinue }
  }
}
New-Item -ItemType Directory -Path artifacts/portable -Force
Copy-Item -LiteralPath src-tauri/target/release/PromptWakeup.exe -Destination artifacts/portable/PromptWakeup.exe
Copy-Item -LiteralPath docs/distribution.md -Destination artifacts/portable/distribution.md
$hash = (Get-FileHash artifacts/portable/PromptWakeup.exe -Algorithm SHA256).Hash.ToLower()
"$hash  PromptWakeup.exe" | Set-Content artifacts/portable/SHA256SUMS.txt -Encoding ascii
"$mode; requires Microsoft Edge WebView2 Evergreen Runtime" | Set-Content artifacts/portable/BUILD_MODE.txt
$name = "PromptWakeup-windows-x64-$mode.zip"
Compress-Archive -Path artifacts/portable/* -DestinationPath "artifacts/$name"
$hash = (Get-FileHash "artifacts/$name" -Algorithm SHA256).Hash.ToLower()
"$hash  $name" | Set-Content "artifacts/$name.sha256" -Encoding ascii
