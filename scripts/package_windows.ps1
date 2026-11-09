$ErrorActionPreference = 'Stop'

function Assert-ExitCode([string]$message) {
    if ($LASTEXITCODE -ne 0) { throw $message }
}
function Get-SigningTool {
    $tool = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe' |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (!$tool) { throw 'signtool not found' }
    return $tool.FullName
}
function Remove-SigningCredentials([string]$path, $certificate) {
    Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
    if ($certificate) {
        Remove-Item -LiteralPath ('Cert:\CurrentUser\My\' + $certificate.Thumbprint) -ErrorAction SilentlyContinue
    }
}
function Sign-Executable {
    $path = Join-Path $env:RUNNER_TEMP 'prompt-wakeup.pfx'
    $certificate = $null
    try {
        [IO.File]::WriteAllBytes($path, [Convert]::FromBase64String($env:CERTIFICATE))
        $password = ConvertTo-SecureString $env:CERTIFICATE_PASSWORD -AsPlainText -Force
        $certificate = Import-PfxCertificate -FilePath $path -CertStoreLocation Cert:\CurrentUser\My -Password $password
        $tool = Get-SigningTool
        & $tool sign /sha1 $certificate.Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 src-tauri/target/release/PromptWakeup.exe
        Assert-ExitCode 'Signing failed'
        & $tool verify /pa src-tauri/target/release/PromptWakeup.exe
        Assert-ExitCode 'Signature verification failed'
    } finally { Remove-SigningCredentials $path $certificate }
}
function Get-BuildMode {
    if (!$env:CERTIFICATE) { return 'development-unsigned' }
    Sign-Executable
    return 'signed'
}
function Write-Checksum([string]$path, [string]$destination) {
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower()
    "$hash  $([IO.Path]::GetFileName($path))" | Set-Content -LiteralPath $destination -Encoding ascii
}

$mode = Get-BuildMode
New-Item -ItemType Directory -Path artifacts/portable -Force | Out-Null
Copy-Item -LiteralPath src-tauri/target/release/PromptWakeup.exe -Destination artifacts/portable/PromptWakeup.exe
Copy-Item -LiteralPath docs/distribution.md -Destination artifacts/portable/distribution.md
Write-Checksum 'artifacts/portable/PromptWakeup.exe' 'artifacts/portable/SHA256SUMS.txt'
"$mode; requires Microsoft Edge WebView2 Evergreen Runtime" | Set-Content artifacts/portable/BUILD_MODE.txt
$name = "artifacts/PromptWakeup-windows-x64-$mode.zip"
Compress-Archive -Path artifacts/portable/* -DestinationPath $name
Write-Checksum $name "$name.sha256"
