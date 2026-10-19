# Builds a NightfirePort release: just NightfirePort.exe (the launcher, and the game with --game; the user, 2026-10-19:
# "no reason to zip when we just have an exe"), built with the release version (NF_RELEASE_VERSION — the self-update
# compares against it), the commit (NF_GIT_HASH, for bug reports) and the licence notices (NF_NOTICES_FILE, baked in
# by crates/nf-view/build.rs; the launcher writes them to %APPDATA%\NightfirePort\THIRD_PARTY_NOTICES.txt). No game
# data: players supply their own disc image.
#
#   powershell -File tools\release\make-release.ps1 -Version 0.1.0 [-Out <dir>] [-Target <cargo target dir>] [-BridgeZip]
#
# Writes <Out>\NightfirePort.exe and NightfirePort.exe.sha256 (the launcher verifies the download with it).
# -BridgeZip also writes NightfirePort-v<Version>-windows.zip (+ .sha256) holding just the exe: the only asset the
# v0.1.0–v0.1.2 launchers know, so the one bridge release (v0.1.3) moves them onto the exe-only updater.
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$Out = "",
    [string]$Target = "",
    [switch]$BridgeZip
)
$ErrorActionPreference = "Stop"
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version must look like 0.1.0 (got '$Version')" }
$ws = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
if (-not $Out) { $Out = Join-Path $ws "target\release-out" }
if (-not $Target) { $Target = Join-Path $ws "target\release-build" }
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force $Out | Out-Null

$notices = Join-Path $Out "THIRD_PARTY_NOTICES.txt"
& python -I (Join-Path $PSScriptRoot "notices.py") $ws $notices movies
if ($LASTEXITCODE -ne 0) { throw "notices failed" }

$env:NF_RELEASE_VERSION = $Version
$env:NF_GIT_HASH = (git -C $ws rev-parse HEAD).Trim()
$env:NF_NOTICES_FILE = $notices
$env:CARGO_TARGET_DIR = $Target
# The C runtime linked in (no VCRUNTIME140.dll import): a lone exe must start on a PC without the Visual C++
# runtime, and a stray DLL of that name beside it (often Downloads) must not load into us. The other hijack-prone
# imports are delay-loaded (crates/nf-view/build.rs); check-imports.py verifies the result.
$env:RUSTFLAGS = "-C target-feature=+crt-static"
& cargo build --release -p nf-view --bins --features movies --manifest-path (Join-Path $ws "Cargo.toml")
$code = $LASTEXITCODE
Remove-Item Env:NF_RELEASE_VERSION, Env:NF_GIT_HASH, Env:NF_NOTICES_FILE, Env:RUSTFLAGS
if ($code -ne 0) { throw "cargo build failed" }
& python -I (Join-Path $PSScriptRoot "check-imports.py") (Join-Path $Target "release\NightfirePort.exe")
if ($LASTEXITCODE -ne 0) { throw "the exe imports a DLL that can be hijacked from its folder" }

$exe = Join-Path $Out "NightfirePort.exe"
Copy-Item (Join-Path $Target "release\NightfirePort.exe") $exe
Remove-Item $notices
function Write-Sha($file) {
    $hash = (Get-FileHash -Algorithm SHA256 $file).Hash.ToLower()
    "$hash  $(Split-Path -Leaf $file)" | Set-Content -Encoding ascii -NoNewline "$file.sha256"
    $hash
}
$hash = Write-Sha $exe
Write-Host "release $Version -> $exe ($([math]::Round((Get-Item $exe).Length / 1MB, 1)) MB, sha256 $hash)"

if ($BridgeZip) {
    $zip = Join-Path $Out "NightfirePort-v$Version-windows.zip"
    # The old updater unpacks the zip's root over the install folder: the exe alone at the root.
    & "$env:SystemRoot\System32\tar.exe" -a -c -f $zip -C $Out NightfirePort.exe
    if ($LASTEXITCODE -ne 0) { throw "zip failed" }
    $zh = Write-Sha $zip
    Write-Host "bridge zip -> $zip (sha256 $zh)"
}
