# Builds a NightfirePort release zip: the launcher (NightfirePort.exe) and the game (NightfirePort-game.exe), built with the
# release version baked in (NF_RELEASE_VERSION — the launcher's self-update compares against it), plus README.txt and
# the licence notices. No game data: players supply their own disc image.
#
#   powershell -File tools\release\make-release.ps1 -Version 0.1.0 [-Out <dir>] [-Target <cargo target dir>]
#
# Writes <Out>\NightfirePort-v<Version>-windows.zip and the .sha256 beside it (the launcher verifies the zip with it).
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$Out = "",
    [string]$Target = ""
)
$ErrorActionPreference = "Stop"
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version must look like 0.1.0 (got '$Version')" }
$ws = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
if (-not $Out) { $Out = Join-Path $ws "target\release-out" }
if (-not $Target) { $Target = Join-Path $ws "target\release-build" }
New-Item -ItemType Directory -Force $Out | Out-Null

$env:NF_RELEASE_VERSION = $Version
$env:CARGO_TARGET_DIR = $Target
& cargo build --release -p nf-view --bins --features movies --manifest-path (Join-Path $ws "Cargo.toml")
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
Remove-Item Env:NF_RELEASE_VERSION

$name = "NightfirePort-v$Version-windows"
$stage = Join-Path $Out $name
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item (Join-Path $Target "release\NightfirePort.exe") $stage
# The game ships as NightfirePort-game.exe (the user's name, 2026-10-19); the launcher looks for it first.
Copy-Item (Join-Path $Target "release\nf-view.exe") (Join-Path $stage "NightfirePort-game.exe")
(Get-Content (Join-Path $PSScriptRoot "README.txt") -Raw).Replace("{VERSION}", $Version) | Set-Content -Encoding utf8 (Join-Path $stage "README.txt")
Copy-Item (Join-Path $ws "THIRD_PARTY_RERAC_LICENSE.txt") $stage
& python -I (Join-Path $PSScriptRoot "notices.py") $ws (Join-Path $stage "THIRD_PARTY_NOTICES.txt") movies
if ($LASTEXITCODE -ne 0) { throw "notices failed" }

$zip = Join-Path $Out "$name.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
# The zip holds the files at its root (the updater unpacks it straight over the install folder).
& "$env:SystemRoot\System32\tar.exe" -a -c -f $zip -C $stage *
if ($LASTEXITCODE -ne 0) { throw "zip failed" }
$hash = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
"$hash  $name.zip" | Set-Content -Encoding ascii -NoNewline "$zip.sha256"
Write-Host "release $Version -> $zip ($([math]::Round((Get-Item $zip).Length / 1MB, 1)) MB, sha256 $hash)"
