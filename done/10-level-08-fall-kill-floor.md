## Your task: `level-08-fall-kill-floor`, falling off 07000008 ends the mission like the game

`crates/nf-view/tests/interactables_07000008_live.rs` `the_kill_floor_ends_the_fall` (~line 844) is #[ignore]d: after a fall the death NIS holds the body at y 17.56, above the y −24.10 kill floor, so the end-to-end fall never reaches the mission fail. The mid-path is pinned by `hurt_07000008_live`. Find the death NIS's terminal event and what it does in the game (mission fail / restart path) [M: the death-NIS event handler and the fail writer; D the NIS stream], port it so the hold ends the way the game's does, and un-ignore the test. Run interactables_07000008_live, hurt_07000008_live, the death-restart tests and walkreal_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-fall-kill-floor`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-02T04:51Z

---
## Report (done, 2026-11-02T06:03Z)


---
## Report from the worker (done, 2026-11-02T06:03Z)

# level-08-fall-kill-floor — the fall off 07000008 ends the mission like the game

Branch `s/level-08-fall-kill-floor`, from origin/master 3628fce7. Sandbox 04:57–06:56 UTC.

## What the task asked, and what the evidence showed

The task: `interactables_07000008_live::the_kill_floor_ends_the_fall` (~line 844) is `#[ignore]`d
because the death NIS holds the body at y 17.56, above the y −24.10 kill floor, so the end-to-end
fall "never reaches the mission fail"; the fix would be to find the death NIS's terminal event and
port it.

**The terminal event is already found, already ported, and already pinned — by the
996-decomp-audit task (`s/996-level-08-decomp-audit`, commit b1ae8e11, merged to master).** The
chain, with its evidence:

- The death NIS (SP #307, script 0x06000991 — `Script_IsDeathNIS` 0x1dae60 [M nis.rs]) takes the
  body when the plunge TouchOnce fires (ch 0x7C [D #54]).
- **The NIS's OWN stream is the terminal event**: stream 6 carries `Wait(89)` then event 0xD with
  params (0x62, 1) at the script's tick 89 [D 0x06000991 — the script-channel census row for
  07000008, `census_channel_events`/`script_channel_events_live`; decoded on the real walk by
  NF_SCRIPTLOG per the audit]. Event 0xD is the switch-channel write [M `Script_EventHandler`
  0x1d94c0].
- `Mission_Update` 0x182c60 reads ch 0x62 → MissionState 3, to_end 0x7000048 [M 0x182c60:21-43],
  the 1.0 s TimeOut → the 0x40000042 fail menu [M 0x182c60:58-81].
- The fail menu's Restart row (`P_ENDMISSION_Handler` 0x216618:19-21) queues the WARM reload of
  the CURRENT level + `Player_RamLoad(Bond, 1)` [M].
- **The body hold is the game's own law, not a port gap**: the NIS's event 4 disables the player
  [M `Script_EventHandler` 0x1d94c0:119; `Player_Update` 0x193600:80-89 returns on the disabled
  byte] and the stream carries no event 6, so `Script_Free`'s 0x100-gated re-enable never runs
  [M 0x1da880:44-49].
- Through that path the y −24.10 kill floor stays SILENT: the held body never descends past the
  NIS's fall floor (≈17.5 — the hold at y 17.56), 41 units above the quads [D #42/#43/#44 y
  −24.10]. The kill floor serves falls the plunge NIS does not catch (those mid-paths are
  `hurt_07000008_live`'s pins; the row/who-mask/touch laws are hurt.rs's unit tests).

`decomp_audit_07000008_live::the_f5_fall_death_nis_writes_0x62_and_opens_the_fail_menu` already
pinned the whole chain on the real input path — the same walk-off from the same platform spot —
and is green on master. Its final assertion is `hurt_hits == 0` ("the held body never reaches the
y −24.1 quads — the NIS path needs no kill floor").

## Why the ignored test was wrong, and what I changed

`the_kill_floor_ends_the_fall` asserted `hurts.iter().any(|h| h.hits > 0)` — "the fall reached
the kill floor". On the very same real walk-off that the decompile-backed audit test pins, that
contradicts the game's law above (the kill floor cannot fire: geometry [D] + the disable law
[M]). Per the task rule ("if an existing test contradicts the decompile, the TEST is wrong: fix
it, with the address"), I fixed the TEST — no port code change was needed:

- Renamed to `the_fall_off_the_platform_ends_the_mission_through_the_death_nis`, un-ignored, and
  re-aimed at the proven law: the real walk off the boarding platform → the plunge TouchOnce (ch
  0x7C) → the death NIS → **its own terminal event 0xD (0x62, 1) at tick 89 [D 0x06000991 s6]** →
  ch 0x62 → MissionState 3 [M 0x182c60:21-43] → the sim's `auto_restart` stand-in answers the
  fail menu's Restart row → `level_exit = Some(0x0700_0008)` (the CURRENT level), warm,
  `level_exit_restart` [M 0x216618:19-21] → and the sister laws: the body still holds above the
  kill floor (y > −24.1) and the kill floor's total hits stay 0 through the path.
- Corrected the two stale notes that carried the old (pre-996-audit) reading:
  `docs/levels/07000008.md` §7's 0xe4 row ("the end-to-end fall is BLOCKED…") and §"The kill
  floor, end to end — blocked one step short" (now "the NIS's own stream event ends the fall").
  `hurt_07000008_live`'s header already carried the same correction ("The old 'the port has no
  death after a death NIS' note in this file was WRONG").

## How it was verified

All on the real input path (`Sim::step_bond` inputs only; the platform-edge teleport is the
hurt test's documented spot), fresh builds against master 3628fce7, `--test-threads=1`:

| suite | result |
|---|---|
| decomp_audit_07000008_live | 1 passed, 0 failed |
| interactables_07000008_live | **14 passed, 0 failed, 1 ignored** (the pre-existing controlroom-glass wedge, unrelated) |
| hurt_07000008_live | 2 passed, 0 failed |
| fix_death_restart_live | 8 passed, 0 failed |
| fix_restart_target_live | 2 passed, 0 failed |
| fix_restart_health_live | 2 passed, 0 failed |
| fix_death_retry_boost_live | 4 passed, 0 failed |
| walkreal_07000008_live | 3 passed, 0 failed (62 s) |
| **nf-game `--tests`, full** | **99 binaries: 1034 passed, 0 failed, 9 ignored** (fresh vs the master merge's drone.rs change) |

## What's left

- Nothing for the task's goal: the fall ends the mission like the game, pinned twice on the real
  input path; the death-restart family and the level's walk are green.
- The fast suite (`tools/test/suite.py`) did NOT complete — see the deviation below.

## Suite deviation (the driver's "run the fast suite" step could not be met)

- Background jobs are killed between tool calls in this sandbox: the suite died within ~5 s of
  its `nohup` launch (suite.out created, process gone), matching three measurements in the
  previous session here. Foreground chunks are the only way to build.
- The foreground full build step includes nf-view's ~550 example targets; master's merge
  (d699b536 → 3628fce7) touched `nf-game/src/drone.rs`, which fingerprint-stales all 918 cached
  nf-view test binaries at once. Disk hit 100% twice this session (the strip+prune cycle
  recovered it to ~800 MB) — the full relink cannot fit the ~2 h window here (also measured in
  the previous session).
- The equivalent coverage actually run is the table above: every test my change touches, plus
  nf-game's complete test suite, all fresh against the final tree. nf-formats/nf-audio are
  untouched by this task; their run attempt produced no output within its timeout (likely a
  rebuild stall on the full disk) and was not retried — the previous merged sessions' suites
  cover them.

## Live checks for the main agent

None required — the chain is pinned by tests on the real input path. One optional confirm, only
if convenient: in PCSX2, walk off the gondola boarding platform and check that at the death NIS's
end the screen shows the held body with the fail menu up (the port's `--bond 111.4,17.56,-136.89`
hold pose + the 0x40000042 menu), matching the game's law above.

## Usage

not visible to me
