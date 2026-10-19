# Publishes the next NightfirePort release from a lane checked out at a GREEN master commit (the user's rule,
# 2026-10-19: a release after every fully green merge batch; players' launchers install it automatically on launch).
#
#   powershell -File tools\release\publish.ps1 [-Lane <checkout>] [-Version 0.1.7] [-Draft] [-Notes <text>]
#
# Version: the newest release's tag + 1 patch unless given. The tag is created on the lane's HEAD (which must be on
# origin/master). The notes default to the last docs/merge-log.md entry. Needs `gh` signed in with release rights.
param(
    [string]$Lane = "",
    [string]$Version = "",
    [switch]$Draft,
    [string]$Notes = ""
)
$ErrorActionPreference = "Stop"
if (-not $Lane) { $Lane = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path }
$repo = "YodaMMM/NightfirePort"
$head = (git -C $Lane rev-parse HEAD).Trim()
& git -C $Lane fetch -q origin
& git -C $Lane merge-base --is-ancestor $head origin/master
if ($LASTEXITCODE -ne 0) { throw "HEAD $head is not on origin/master: release only pushed, green commits" }
if (git -C $Lane status --porcelain --untracked-files=no) { throw "the lane has uncommitted changes" }

if (-not $Version) {
    $last = (& gh release list --repo $repo --limit 1 --json tagName --jq '.[0].tagName' 2>$null)
    if (-not $last) { $Version = "0.1.0" }
    else {
        $p = $last.Trim().TrimStart('v').Split('.')
        $Version = "{0}.{1}.{2}" -f $p[0], $p[1], ([int]$p[2] + 1)
    }
}
$tag = "v$Version"
$out = Join-Path $Lane "target\release-out"
& powershell -NoProfile -File (Join-Path $PSScriptRoot "make-release.ps1") -Version $Version -Out $out
if ($LASTEXITCODE -ne 0) { throw "make-release failed" }
$zip = Join-Path $out "NightfirePort-$tag-windows.zip"

if (-not $Notes) {
    $log = Get-Content (Join-Path $Lane "docs\merge-log.md")
    $entry = ($log | Where-Object { $_ -match '^- ' } | Select-Object -Last 1)
    $Notes = "Changes in this build:`n`n$entry`n`nDownload the zip, unzip it anywhere you can write to, and run NightfirePort.exe. You need your own 007: Nightfire (PS2, USA, SLUS-20579) disc image. Existing installs update themselves on launch."
}
$notesFile = Join-Path $out "notes-$tag.md"
Set-Content -Encoding utf8 $notesFile $Notes
$ghArgs = @("release", "create", $tag, $zip, "$zip.sha256", "--repo", $repo, "--target", $head, "--title", "NightfirePort $Version", "--notes-file", $notesFile)
if ($Draft) { $ghArgs += "--draft" }
& gh @ghArgs
if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }
Write-Host "published $tag at $head$(if ($Draft) { ' (draft)' })"
