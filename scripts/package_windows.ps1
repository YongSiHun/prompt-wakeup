$ErrorActionPreference = 'Stop'

function Write-Checksum([string]$path, [string]$destination) {
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower()
    "$hash  $([IO.Path]::GetFileName($path))" | Set-Content -LiteralPath $destination -Encoding ascii
}

New-Item -ItemType Directory -Path artifacts/portable -Force | Out-Null
Copy-Item -LiteralPath src-tauri/target/release/PromptWakeup.exe -Destination artifacts/portable/PromptWakeup.exe
Copy-Item -LiteralPath docs/distribution.md -Destination artifacts/portable/distribution.md
Write-Checksum 'artifacts/portable/PromptWakeup.exe' 'artifacts/portable/SHA256SUMS.txt'
$name = 'artifacts/PromptWakeup-windows-x64.zip'
Compress-Archive -Path artifacts/portable/* -DestinationPath $name
Write-Checksum $name "$name.sha256"
