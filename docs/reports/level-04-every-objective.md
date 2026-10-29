# level-04-every-objective — 07000004 (Double Cross part 4) completable end to end on real input

## What the level's mission data lists (the coverage list, from the disc)

The mission data for 07000004 lists **exactly ONE objective record**. Decoded straight from the
EE snapshot (file offset = EE address):

- `ee_snapshot.bin` 0x2a4350 + row 7×40 (level 0x07000004, mission 0x07000001, part 3) → objective
  table pointer **0x2a3f60, count 1** [L]; matches the `MissionData` symbol at ACTION.ELF 0x2a4350
  [M] and `crates/nf-game/src/mission/c07000004.rs`.
- The one record: **obj0 text 0x0400000d "Defeat the assassin", fail 0xffffffff (none), hint
  0x04000015, condition channel 0x54, init 0, enable channel 0x53, flags 0** [L 0x2a3f60].

## Which objectives the current tests already cover — all of them

`walk_the_full_mission_start_to_finish` (crates/nf-view/tests/walkreal_07000004_live.rs) already
walks the WHOLE mission from a fresh load on real input (`step_bond` movement/aim/fire/use; no
teleports, no channel/state pokes), in the order a player does it, and asserts every step:

1. fresh load; the intro SP #3 + story scene SP #2 settle (AI re-enabled, ch 0x53 written) [D
   scripts 0x06000083/0x06000082 stream 9];
2. obj0 announced through the real monitor (state 3, enable arm [M Mission_MonitorObjectives
   0x1827d8:55-67]);
3. the armament walk: StormUzi pickup #7 → weapon 18 owned [M Pickup_Handler 0x18e650 type-0 arm];
   BodyArmourHeavy #9 → armour raised, case consumed [M 0x18e650:220-225, 338-348];
4. the kill on the real fire path → ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660];
5. OUT trigger #46 mirrors ch 0x54 → ch 100 [D list [0x64]; M 0x1b1a78 case 5:147-160];
6. obj0 folds complete (state 4) through the real monitor [M 0x1827d8:78-85];
7. the "Mission Complete" banner (case 6, LevelToEndTo 0x0730004 [M 0x182c60:85-122]) → case 7's
   hold → case 4 → **the level exit fires**: `mission_end = Some(0x0730004)`, MissionState 6
   [M 0x182c60:65-76] — the mission layer IS this level's exit (no 0xE8 trigger exists on the
   level [D placement probe]).

Sisters pin the negative (without the kill the mission stays open, obj0 never leaves state 3), the
channel-level assassination, the full outro sequence (the GS_PausePlayer hold, the movie wrap, the
results page) and the results/stats/dossier tallies. **There are no further objectives to leg** —
the level is a one-objective mission, and that objective is covered complete, ending on the exit.

## What this task changed

- `crates/nf-view/tests/walkreal_07000004_live.rs`: pinned the whole objective list in code —
  `assert_eq!(mission::LEVELS[row].objectives.len(), 1, ...)` with the [L] cite, plus the header
  coverage note recording the list above. No behaviour change.
- `docs/WORKLOG.md`: this task's entry (branch-opened + results).

## How it was verified

- `walkreal_07000004_live`: **5 passed, 0 failed, 0 ignored (4.0 s)** on this branch, NF_ISO set,
  real input path, fresh load per test.
- Independent re-decode of the objective record from `work/ee_snapshot.bin` (the python decode in
  the WORKLOG) — matches the port's `c07000004.rs` byte for byte.
- The all-levels census test (`mission::every_level_objectives_complete_through_the_monitor_and_
  gate_ch_100`, in the suite) re-proves the monitor + ch-100 gate for every level's record set
  including this one.

## The FULL suite (fast runner, chunked foreground)

This sandbox kills background jobs between tool calls (confirmed again this session; `nohup … &`
died right after launch), so the suite ran via `tools/test/chunk_suite.py` in foreground chunks
(state file `suite_l04_state.json`, per-binary results recorded as each binary finishes).

- At the 01:08 UTC checkpoint: **199/888 binaries recorded — 1564 passed, 4 failed, 45 ignored**.
- The 4 failures, with evidence:
  - `disc_golden`: **3 failed** — exactly the 3 the suite rules allow on the slim on-foot ISO.
  - `identity_disc::the_real_disc_passes_the_identity_check`: pins the FULL retail disc identity
    (1,191,072 sectors, docs/build-identity.md); the sandbox image is the trimmed on-foot ISO
    (518,435 sectors — the assert's own `left:` value). Same slim-ISO environment limitation as
    the disc_golden trio, not a port gap. Left untouched per "never weaken a disc-backed test".
- The tally keeps climbing while the sandbox window lasts; the final number for THIS session is
  the last `chunk_suite` line quoted below in "Suite total (final)".

## Port gaps found

None on this level: every blocker the task lists (doors, interactables, missing models, scripted
events, guards) is already cleared on master — the level has zero doors/switches/locks/turrets etc.
(docs/levels/07000004.md interactables census: 70 placements, 0 unresolved, no 0xE8 exit), and the
walk passes end to end on real input. **No ignores were added; no live checks needed.**

## Sandbox notes for the next session

- Background jobs die between tool calls here: use `tools/test/chunk_suite.py` (or my fast-first
  reorder variant, `/home/z/my-project/scripts/chunk_suite_fastfirst.py` — same state schema) in
  foreground chunks of ≤ 585 s.
- The nf-view/nf-audio link step needs `LIBRARY_PATH=/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu`
  (alsa-sys's pkg-config emits only the system `-L`, where the dev symlink `libasound.so` doesn't
  exist; the prefix has it). `PKG_CONFIG_PATH` alone is not enough.
- Disk: deleting stale test executables freed 4.2 GB (`target/release/deps` had 1075 binaries from
  previous sessions/branches).

## What's left

- The remaining suite binaries (mostly the live-test tail) — rerun `chunk_suite.py` until it prints
  ALL DONE on this branch; expect the same 4 environment failures only.
- Nothing else: the objective coverage is complete (1/1) and green.

## Usage

not visible to me (this environment exposes no token, cost or context-window figures to the agent).
