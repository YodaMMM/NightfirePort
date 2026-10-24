# Publishes the next NightfirePort release from a checkout at a GREEN master commit (players' launchers install it automatically on launch).
#
#   powershell -File tools\release\publish.ps1 -NotesFile <notes.md> [-Lane <checkout>] [-Version 0.1.7] [-Draft]
#   (or -Notes <text>; prefer -NotesFile: straight double quotes in -Notes break `powershell -File` argument passing)
#
# Version: the newest release's tag + 1 patch unless given. The tag is created on the lane's HEAD (which must be on
# origin/master). The notes are required: a short plain-English summary of what changed for players, nothing appended. Assets: NightfirePort.exe + .sha256 (v0.1.3 also the bridge zip). Needs `gh` signed in with release rights.
param(
    [string]$Notes = "",
    [string]$NotesFile = "",
    [string]$Lane = "",
    [string]$Version = "",
    [switch]$Draft
)
$ErrorActionPreference = "Stop"
if (-not $NotesFile) { throw "-NotesFile is required: every note item needs a row in the proof.md beside it" }
$Notes = [IO.File]::ReadAllText((Resolve-Path $NotesFile), [Text.Encoding]::UTF8)
if (-not $Notes.Trim()) { throw "release notes are required (-NotesFile)" }
# The proof gate: proof.md beside the notes, one row per note item —
#   - <words from the item> | <evidence path: the before/after real-path proof in the shipped build> | <commit sha>
# Every bold item and every bullet of the notes must contain the words of some row; every evidence path must exist;
# every commit must be on HEAD and NOT already in the previous release (nothing old listed as new).
$proofFile = Join-Path (Split-Path (Resolve-Path $NotesFile)) "proof.md"
if (-not (Test-Path $proofFile)) { throw "no proof file: $proofFile" }
$rows = @(Get-Content $proofFile -Encoding UTF8 | Where-Object { $_ -match '^\s*-\s*(.+?)\s*\|\s*(.+?)\s*\|\s*([0-9a-f]{7,40})\s*$' } | ForEach-Object {
    $m = [regex]::Match($_, '^\s*-\s*(.+?)\s*\|\s*(.+?)\s*\|\s*([0-9a-f]{7,40})\s*$')
    [pscustomobject]@{ Key = $m.Groups[1].Value; Path = $m.Groups[2].Value; Sha = $m.Groups[3].Value } })
$items = @($Notes -split "`r?`n" | Where-Object { $_ -match '^\s*(\*\*.+?\*\*|-\s+\S)' })
$prevTag = (& gh release list --repo "YodaMMM/NightfirePort" --limit 1 --json tagName --jq '.[0].tagName' 2>$null)
$laneForGate = if ($Lane) { $Lane } else { (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path }
$missing = @()
foreach ($it in $items) {
    $row = $rows | Where-Object { $it.Contains($_.Key) } | Select-Object -First 1
    if (-not $row) { $missing += "no proof row: $it"; continue }
    if (-not (Test-Path $row.Path)) { $missing += "evidence missing ($($row.Path)): $it"; continue }
    & git -C $laneForGate merge-base --is-ancestor $row.Sha HEAD
    if ($LASTEXITCODE -ne 0) { $missing += "commit $($row.Sha) not on HEAD: $it"; continue }
    if ($prevTag) {
        & git -C $laneForGate merge-base --is-ancestor $row.Sha $prevTag.Trim()
        if ($LASTEXITCODE -eq 0) { $missing += "commit $($row.Sha) already shipped in $($prevTag.Trim()): $it" }
    }
}
if ($missing.Count) { throw ("proof gate failed:`n" + ($missing -join "`n")) }
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
# The bridge zip goes on v0.1.3 only (the release that moves the v0.1.0–v0.1.2 zip-only launchers onto the exe).
$bridge = ($Version -eq "0.1.3")
$mk = @("-NoProfile", "-File", (Join-Path $PSScriptRoot "make-release.ps1"), "-Version", $Version, "-Out", $out)
if ($bridge) { $mk += "-BridgeZip" }
& powershell @mk
if ($LASTEXITCODE -ne 0) { throw "make-release failed" }
$exe = Join-Path $out "NightfirePort.exe"
$assets = @($exe, "$exe.sha256")
if ($bridge) { $zip = Join-Path $out "NightfirePort-$tag-windows.zip"; $assets += @($zip, "$zip.sha256") }

# The notes are the plain-English summary only.
$notesFile = Join-Path (Split-Path $out) "notes-$tag.md"
[IO.File]::WriteAllText($notesFile, $Notes, (New-Object System.Text.UTF8Encoding $false))  # no BOM (5.1's utf8 adds one)
$ghArgs = @("release", "create", $tag) + $assets + @("--repo", $repo, "--target", $head, "--title", "Nightfire Rust $Version", "--notes-file", $notesFile)
if ($Draft) { $ghArgs += "--draft" }
& gh @ghArgs
if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }
Write-Host "published $tag at $head$(if ($Draft) { ' (draft)' })"
