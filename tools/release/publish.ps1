# Publishes the next NightfirePort release from a lane checked out at a GREEN master commit (the user's rule,
# 2026-10-19: a release after every fully green merge batch; players' launchers install it automatically on launch).
#
#   powershell -File tools\release\publish.ps1 -NotesFile <notes.md> [-Lane <checkout>] [-Version 0.1.7] [-Draft]
#   (or -Notes <text>; prefer -NotesFile: straight double quotes in -Notes break `powershell -File` argument passing)
#
# Version: the newest release's tag + 1 patch unless given. The tag is created on the lane's HEAD (which must be on
# origin/master). The notes are required: a short plain-English summary of what changed for players (the user's choice,
# 2026-10-19 — not the merge log's jargon), nothing appended. Assets: NightfirePort.exe + .sha256 (v0.1.3 also the bridge zip). Needs `gh` signed in with release rights.
param(
    [string]$Notes = "",
    [string]$NotesFile = "",
    [string]$Lane = "",
    [string]$Version = "",
    [switch]$Draft
)
$ErrorActionPreference = "Stop"
if ($NotesFile) { $Notes = [IO.File]::ReadAllText((Resolve-Path $NotesFile), [Text.Encoding]::UTF8) }
if (-not $Notes.Trim()) { throw "release notes are required (-NotesFile or -Notes)" }
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

# The notes are the plain-English summary only (the user: the old install line was pointless).
$notesFile = Join-Path (Split-Path $out) "notes-$tag.md"
[IO.File]::WriteAllText($notesFile, $Notes, (New-Object System.Text.UTF8Encoding $false))  # no BOM (5.1's utf8 adds one)
$ghArgs = @("release", "create", $tag) + $assets + @("--repo", $repo, "--target", $head, "--title", "NightfirePort $Version", "--notes-file", $notesFile)
if ($Draft) { $ghArgs += "--draft" }
& gh @ghArgs
if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }
Write-Host "published $tag at $head$(if ($Draft) { ' (draft)' })"
