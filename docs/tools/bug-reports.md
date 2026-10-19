# Player bug reports

The user's requirement (2026-10-19): players need a way to report bugs that is robust and gives us every detail we
need. The inbox is GitHub Issues (`YodaMMM/NightfirePort`, label `bug-report`).

## The player's side

- **F10, anywhere in the game, menus included** (`App::bug_report`, crates/nf-view/src/bugreport.rs) writes
  `%APPDATA%\NightfirePort\reports\NightfirePort-report-<YYYYMMDD-HHMMSS>.zip`, opens the issue form
  (`.github/ISSUE_TEMPLATE/bug_report.yml`) pre-filled (title, version, level, report id, system + place) and shows
  the zip selected in Explorer. The player describes the problem and drags the zip into the form's "Report file" box.
- **Crashes**: the panic hook (`bugreport::install_crash_hook`) writes the same zip plus `crash.txt` (message and
  backtrace) and a `.pending` marker. The launcher shows "The game crashed…" and a gold **Report crash** link on its
  next start (or right after the game exits with a crash); the link opens the form for that zip.

## What a report holds

| File | Content |
|---|---|
| `report.txt` | `key: value` lines: kind, report id, version + commit (`NF_GIT_HASH`, baked in by the release build), Windows version, GPU + driver, window, fps, difficulty, level, tick, Bond's pose / substate / health / armour, weapon + ammo, the mission globals and every objective state, the `--bond` spot, the front-end page, the online session (slot, net tick, agents, reach) and the MP match settings, the session note |
| `screenshot.png` | the frame at F10 (the F12 capture path) |
| `session.nfr` | the input replay of the current level since it (re)started — `crate::replay` format, header from the Sim before the first tick; capped at one hour (the note says if it was cut) |
| `entry.sav` | the carried state the level started with (`App::entry_save`, a `RamSave`; `bugreport::encode_ram_save`) |
| `settings.toml` | the player's settings |
| `game-log.txt` | the last 3000 lines of `nf-view.log` |
| `crash.txt` | crash reports only |

Personal paths are scrubbed (`%USERPROFILE%`, `<user>`). The zip is stored uncompressed (`bugreport::zip_store`).
Online matches carry their settings and session state but not yet a replayable transcript (local play is fully
replayable).

## Our side

- `NF_ISO=<iso> nf-view --bug-replay <report.zip> [--shot out.png]` loads the level with the report's entry save and
  difficulty, checks the start state against the recording, replays it headless and prints where Bond ends against
  what the report recorded; `--shot` renders Bond's view at the end.
- `NF_ISO=<iso> python -I tools/bugreport/fetch.py [--exe <nf-view.exe>]` lists new open `bug-report` issues,
  downloads every report zip attached to the issue or its comments into `work/bugreports/<issue>/`, runs
  `--bug-replay` on each and writes `summary.txt` (the issue text, report.txt, the replay result). The main agent runs
  it with the merge routine and turns each report into a queue task (or a fix) the same day.

Test aids: `NF_BUG_REPORT_AT=<tick>` takes an F10 report at that level tick; `NF_BUG_REPORT_NO_OPEN` keeps the browser
and Explorer closed; `NF_TEST_PANIC_AT=<tick>` panics on purpose (the crash path).

## Verified (2026-10-19, local)

- F10 at tick 600 of 07000001: a 694 KB zip (screenshot, report.txt, session.nfr, entry.sav); `--bug-replay` matched
  the start state and ended on the recorded spot.
- A moving session (walk / turn / fire / reload, 881 ticks of 07000005 played in the window through `--replay`):
  `--bug-replay` ended on the exact recorded position, yaw and ammo (clip 3, spare 36).
- `NF_TEST_PANIC_AT=300` on 07000005: the crash zip (crash.txt, context, session, entry save) and its `.pending` marker.
