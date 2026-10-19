# Releases and the launcher's self-update

The user's rule (2026-10-19): a public GitHub release goes out after every fully green merge batch, and players'
launchers install it automatically on launch.

## What a release is

`NightfirePort-v<X.Y.Z>-windows.zip` on a GitHub release of `YodaMMM/NightfirePort`, with
`NightfirePort-v<X.Y.Z>-windows.zip.sha256` beside it. The zip holds, at its root:

| File | What |
|---|---|
| `NightfirePort.exe` | the launcher (GUI subsystem): disc image picker, PLAY, self-update |
| `NightfirePort-game.exe` | the game (the `nf-view` build renamed; console subsystem, the launcher starts it hidden; dev builds keep `nf-view.exe`, which the launcher also finds) |
| `README.txt` | player instructions (`tools/release/README.txt`, version filled in) |
| `THIRD_PARTY_NOTICES.txt` | every crate linked into the binaries with its licence files (`tools/release/notices.py`) |
| `THIRD_PARTY_RERAC_LICENSE.txt` | the in-tree rerac code's notice |

No game data: players supply their own disc image. Online play uses join codes (the host's lobby shows one once UPnP / NAT-PMP or STUN reach it). The listing-server address is never baked in (a player may put
`lobby_server = host:port` in `%APPDATA%\NightfirePort\net.txt`).

## Building and publishing

- `tools/release/make-release.ps1 -Version X.Y.Z` builds both binaries with `--features movies` and
  `NF_RELEASE_VERSION=X.Y.Z` (the launcher's version; a build without it is a dev build that never updates), stages the
  files, zips them with the system `tar.exe` and writes the `.sha256`.
- `tools/release/publish.ps1 [-Lane <checkout>]` refuses unless HEAD is on `origin/master` and the tree is clean, takes
  the newest release's tag + 1 patch, runs make-release, and `gh release create`s the tag on HEAD with both files and
  a required plain-English `-Notes` summary (the user's choice) plus the install line. `-Draft` for a draft.

## The self-update (crates/nf-view/src/launcher/update.rs)

On start a release-built launcher: deletes `*.old` leftovers beside it (retried for a few seconds — the previous
launcher may still be exiting); queries `https://api.github.com/repos/YodaMMM/NightfirePort/releases/latest` with the
system `curl.exe`; when the tag is newer, downloads the zip and its `.sha256`, checks the SHA-256
(`nf_formats::sha256`; a corruption check — both files come over HTTPS from the same release, so not a signature),
unpacks with the system `tar.exe` into `%TEMP%\NightfirePort-update\stage-<tag>`, renames each file it replaces to
`<name>.old` (a running exe can be renamed, not overwritten) and copies the new one in (rolled back on any failure),
then starts the new launcher with the same arguments and exits. PLAY shows UPDATING… meanwhile; the footer shows the
version and the update step. Any failure (offline, no write access) leaves the installed version and PLAY works.

Test aids: `NF_UPDATE_API=<url>` replaces the release query (a `file:///…/latest.json` works — curl reads files);
`NF_NO_UPDATE=1` skips the check; `NF_LAUNCHER_AUTOPLAY=1` presses PLAY once any update finished;
`NightfirePort --shot out.png --state updating --iso <path>` renders the updating screen.

## Verified (2026-10-19, local)

v0.1.0 unzipped into an empty folder, clean `%APPDATA%`, `NF_UPDATE_API` → a v0.1.1 `latest.json` with `file://` assets,
autoplay: the launcher updated both exes (hashes = v0.1.1's), restarted, removed the `.old`, started the game (intro
movies, title screen) and closed. Offline (`NF_UPDATE_API` unreachable): v0.1.0 kept, the game started.
