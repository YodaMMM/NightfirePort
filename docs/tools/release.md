# Releases and the launcher's self-update

The user's rule (2026-10-19, revised the same evening — "Might be putting releases out too frequently"): green
batches merge to master all day, and **one public GitHub release goes out each evening (~9 pm MST)** with everything
that passed since the last one. An urgent fix for something badly broken may go out sooner. Players' launchers
install it automatically on launch.

## What a release is

Just `NightfirePort.exe` on a GitHub release of `YodaMMM/NightfirePort`, with `NightfirePort.exe.sha256` beside it
(the user, 2026-10-19: "no reason to zip when we just have an exe"). The exe is the one program (GUI subsystem):
double-clicked, it's the launcher (disc picker, PLAY, self-update, Report crash); PLAY runs the same file as
`NightfirePort.exe <iso> --game`, the game (`nf_view::run`), with its output sent to `nf-view.log`.

- The licence notices are baked into the exe (`crates/nf-view/build.rs`, `NF_NOTICES_FILE` from
  `tools/release/notices.py`, plus the in-tree rerac notice). The launcher writes them to
  `%APPDATA%\NightfirePort\THIRD_PARTY_NOTICES.txt`.
- The zip era (v0.1.0–v0.1.2) shipped `NightfirePort-v<X.Y.Z>-windows.zip` with README.txt and the notices.
  **v0.1.3 is the bridge**: it also carries that zip, holding just the exe, because the old launchers only know the
  zip asset. Every release after it is the exe alone.
- v0.1.0 shipped the game as a second `NightfirePort-game.exe`; the launcher deletes that file on start.

No game data: players supply their own disc image. Online play uses join codes (the host's lobby shows one once UPnP / NAT-PMP or STUN reach it). The listing-server address is never baked in (a player may put
`lobby_server = host:port` in `%APPDATA%\NightfirePort\net.txt`).

## Building and publishing

- `tools/release/make-release.ps1 -Version X.Y.Z [-BridgeZip]` writes the notices, builds `NightfirePort.exe` with
  `--features movies`, `NF_RELEASE_VERSION=X.Y.Z` (a build without it is a dev build that never updates),
  `NF_GIT_HASH` and `NF_NOTICES_FILE`, and writes the `.sha256` (and the bridge zip + its `.sha256`).
- `tools/release/publish.ps1 [-Lane <checkout>]` refuses unless HEAD is on `origin/master` and the tree is clean,
  takes the newest release's tag + 1 patch, runs make-release (the bridge zip on 0.1.3 only), and `gh release create`s
  the tag on HEAD with the files and the required plain-English summary (`-NotesFile notes.md`; `-Notes` text breaks on straight double quotes), with nothing appended. `-Draft`
  for a draft. Mission names in the notes come from the mission-select table (`CAMPAIGN` in menu.rs), never from
  memory or docs titles.

## The self-update (crates/nf-view/src/launcher/update.rs)

On start a release-built launcher:
1. Deletes the `.old` leftovers beside it: only its own exe name and the zip era's file names, never other files
   (a lone exe may sit in Downloads). This is retried for a few seconds, because the previous launcher may still be
   exiting.
2. Queries `https://api.github.com/repos/YodaMMM/NightfirePort/releases/latest` with the system `curl.exe`.
3. When the tag is newer, downloads `NightfirePort.exe` and its `.sha256` into
   `%TEMP%\NightfirePort-update\stage-<tag>` (under the running exe's own file name, so a renamed exe stays renamed)
   and checks the SHA-256 (`nf_formats::sha256`). This is a corruption check: both files come over HTTPS from the
   same release, so it isn't a signature.
4. Renames the running exe to `<name>.old` (a running exe can be renamed, not overwritten) and copies the new one in,
   rolling back on any failure.
5. Starts the new launcher with the same arguments and exits.

PLAY shows UPDATING… meanwhile; the footer shows the version and the update step. Any failure (offline, no write
access) leaves the installed version in place and PLAY works.

Test aids: `NF_UPDATE_API=<url>` replaces the release query (a `file:///…/latest.json` works — curl reads files);
`NF_NO_UPDATE=1` skips the check; `NF_LAUNCHER_AUTOPLAY=1` presses PLAY once any update finished;
`NightfirePort --shot out.png --state updating --iso <path>` renders the updating screen.

## Verified (2026-10-19, local)

v0.1.0 unzipped into an empty folder, clean `%APPDATA%`, `NF_UPDATE_API` → a v0.1.1 `latest.json` with `file://` assets,
autoplay: the launcher updated both exes (hashes = v0.1.1's), restarted, removed the `.old`, started the game (intro
movies, title screen) and closed. Offline (`NF_UPDATE_API` unreachable): v0.1.0 kept, the game started.

Single exe (same day): the published v0.1.0 zip (two exes) downloaded from GitHub and unzipped, `NF_UPDATE_API` → a
local one-exe v0.1.1: the launcher updated, deleted `NightfirePort-game.exe`, restarted, and PLAY ran
`NightfirePort.exe <iso> --game` (intro movies in `nf-view.log`); the launcher process closed.
