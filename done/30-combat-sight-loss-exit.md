## Your task: `combat-sight-loss-exit`, the combat router's sight-loss exit to NoOpponent, without stalling the walks

On master the router head in drone.rs (the `from` match on COMBAT / COMBAT_OUT_OF_RANGE / COMBAT_NO_SIGHT / AIM_*_FIRE with `!self.seen && self.unseen_ticks > 15`) is disabled with `if false`, and `(NO_OPPONENT, msg::UPDATE)` returns to Attack on `has_opponent(frame)` alone. With the exit on and the NO_OPPONENT rule `(team == 2 && has_opponent) || (team != 2 && seen)`, these broke: ai_missing_states_live (no_opponent_waits_then_attacks_when_one_exists: stuck in 0x60), fix_searchlights_guards_live, fix_searchlight_alarm_live, abseil_elevator_live, npc_doors_live (06 chase door), bathhouse_voices_live, fix_finale_live, walkreal_07000001_live.

1. Port the router head from [M 0x148338:19-31] with the game's own hold: `HasOpponent` 0x14ad58 reads the +0x238 stamp (30 frames), not the port's 15-tick `unseen_ticks` stand-in. Cite where +0x238 is written.
2. NoOpponent 0x60 case 3 [M 0x1650a0:30-49]: what re-points +0x170 for a team-1 guard. Port that writer rather than a `seen` stand-in.
3. Every test above must pass with the change on, plus the 07000001 hall-guard case the exit was for: a sight-lost guard must not sit in 0x5c forever (walkreal_07000001_live and escort_follow_07000001_live green).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/combat-sight-loss-exit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-24T08:29Z

---
## Report (done, 2026-10-24T09:33Z)


---
## Report from the worker (done, 2026-10-24T09:33Z)

# combat-sight-loss-exit — the combat router's sight-loss exit to NoOpponent, without stalling the walks

Task: on master the router head in drone.rs (the `from` match on COMBAT / COMBAT_OUT_OF_RANGE /
COMBAT_NO_SIGHT / AIM_*_FIRE with `!self.seen && self.unseen_ticks > 15`) is disabled with `if false`,
and `(NO_OPPONENT, msg::UPDATE)` returns to Attack on `has_opponent(frame)` alone. With the exit on and
the NO_OPPONENT rule `(team == 2 && has_opponent) || (team != 2 && seen)`, eight tests broke. All nine
named tests plus the 07000001 hall-guard case are now green with the router head ported the game's own
way: the gate is the +0x170 pointer and the +0x238 stamp — sight is never read there.

## What the game does (evidence)

1. **The router head is the pointer/stamp gate, not sight** [M DroneFunc_CombatState 0x148338:19-31]:
   `+0x170 == 0` with no fresh `+0x238` hold returns NoOpponent 0x60 — `(stamp == 0) || (stamp + 0x1e <
   GameState._52_4_)` kills the hold. `NDrone2_HasOpponent` 0x14ad58:8-15 is the same check
   (+0x170 != 0 → 1; else +0x238 != 0 && frame <= stamp + 0x1e → 1; else 0). The port's
   `unseen_ticks > 15` block was a sight-based stand-in for this gate — deleted.
2. **Where +0x238 is written** [M]: `DroneVision_HaveOpponentSight` 0x175ef0:66-67 stamps
   `+0x238 = GameState._52_4_` whenever the opponent (+0x170) is present, behind the reaction gate
   (:20-25: `+0x270 >= DroneFunc_ReactionTime`), reached from `DroneVision_ProcessDroneSight`
   0x176a28:216-231 on every opponent-visible tick — ANY team. Readers: HasOpponent 0x14ad58:11-14,
   SeenAndAttacking 0x145150:17,31-32 (the music window), BotGlobal 0x16e060:504.
3. **What re-points +0x170 for a team-1 guard** [M]: `NDrone2_FindOpponent` 0x1424e0:100-121 — the SP
   `+0x44 != 2` branch calls `NDrone2_SetOpponent` 0x1422d0:19 with the player object EVERY tick (from
   `NDrone2_ControlSTANDARD` 0x1490f0:167), zeroing it only when `PlrStat_OkToUpdate` fails (:106-109)
   or the player object is class 3 with health <= 0 (:111-114) or class 0x12 (:116-118). So a team-1
   guard's +0x170 is "Bond alive", NOT "Bond seen" — the NoOpponent exit [M 0x1650a0:27-31 case 3,
   :37-47 case 0xc, both `+0x170 != 0` → SetState(0x56), never the stamp] fires for them only when the
   PLAYER is dead. The old `(team != 2 && seen)` exit rule parked unseening guards in 0x60 — that was
   the break. For team-2 drones the pointer IS sight-cleared [M 0x1424e0:148-156 CanSeeObject fails →
   SetOpponent(0)], which the port's team-2 re-pick already models (drone.rs `team2_opponent_picks`
   comment "[M 0x1424e0:148-156] a held opponent survives the tick only while visible").
4. **Why the sight-lost 0x5c guard never needed a sight-based exit**: in the game a 0x5c guard whose
   player is alive keeps routing through the router's own moving arms [M 0x148338 case 4 →
   SetCombatMoveAnim / SetAngleToObj / 0x7f / 0x5f], and 0x5f re-routes every second [M 0x164c08 case
   0xc alternates MoveToObject / SetGoalToDroneWithLos] — the port already mirrors those arms, so with
   the pointer-model head the 07000001 hall guard walks instead of freezing.

## The port change (crates/nf-game/src/drone.rs)

- `has_opponent(frame)`: the +0x238 30-frame grace now applies to EVERY team (was team-2 only); team
  1/3 = `bond_alive || grace` [M 0x14ad58:8-15 with the 0x1424e0:100-121 pointer fold].
- `opp_pointer()` (new): the +0x170 pointer alone, no grace — used by the NoOpponent UPDATE/TIMER_C
  exits [M 0x1650a0:27-31, :37-47]; COMBAT_NO_MOVE's exit keeps `has_opponent` [M 0x1643e8:68 calls
  NDrone2_HasOpponent].
- The team-1/3 sight pass stamps `opp_have_frame = frame` behind the reaction gate [M 0x175ef0:66-67];
  team 2 re-stamps in its opponent fold as before.
- The `if false` sight-based head block is deleted; the live first gate `!has_opponent → NO_OPPONENT`
  IS the router head [M 0x148338:19-31] and sits before the in-transition check, as in the decompile.

## Verification

All runs on the final tree (merge of origin/master 26543436 + this change; master had not moved when
re-fetched at 09:29 UTC). `NF_SILENT=1`, `--test-threads=1` per binary, the suite's own env.

The nine named tests, all green with the change on:
- ai_missing_states_live: 7/7 (incl. no_opponent_waits_then_attacks_when_one_exists — the 0x60 hold
  over a dead Bond, then Attack on the next update once the pointer re-points)
- fix_searchlights_guards_live 2/2; fix_searchlight_alarm_live 2/2
- abseil_elevator_live 3/3
- npc_doors_live 6 passed, 1 ignored (the escort-door ignore is a pre-existing hold from
  level-05-complete-2, documented in the test — not touched)
- bathhouse_voices_live 3/3
- fix_finale_live 4/4
- walkreal_07000001_live 3/3 (walk_from_the_spawn_to_the_exit_with_the_contact — the hall guards
  engage, the escort reaches the plate, the exit opens: the "sight-lost guard must not sit in 0x5c
  forever" case)
- escort_follow_07000001_live 1/1 (walking-speed follow bands + the AND plate)

Regression sweep (the change touches shared AI semantics):
- nf-game: the WHOLE crate, all 84 test binaries, chunked: 0 failures (≈420 tests; per-binary result
  lines in the session log; includes ai_live, guard_*, doors_*, mp_*, searchlight_live, sensor_live,
  stuck_guards_sweep).
- nf-view AI/walk subset, 30+ binaries: 0 failures — walkreal 01/02/03(335 s)/05/06/07/08(117 s)/09/
  0a/0c/0d/11/11-route/12/13/14/15/16/1b/41/4a, escort_cover, escort_hide_findrun, stairwell_guards_
  07000011, searchlight_late_spots, fix_searchlight_remainder, fix_guard_weapons_live,
  fix_guard_first_shot_live, fix_guard_misc_live, inert_guards_02, first_sight_flag_live,
  consider_alerted_live, guard_move_checks_live, guard_stuck_sweep_live, fix_ai_remainder_live,
  ai_traces_live, alert_status_live. Every "0 passed, N ignored" line is a pre-existing `#[ignore]`
  in that file (checked).

### Why the full suite is chunked (environment constraint)

This sandbox's rootfs is 9.9G and sat at 92% at session start; the image's lower-overlay files are
unreclaimable (deleting node_modules freed 0 bytes) and this session's builds copy up. The suite build
hit ENOSPC twice (`No space left on device` while creating libnf_view-*.rlib — suite.out holds the
log). Reclaiming the session's stale binaries left ~840M free, which fits per-chunk builds but not the
full four-crate `cargo test --no-run` (≈3.5G of test binaries). So the suite ran as per-crate,
per-chunk `cargo test --offline --release --test a --test b … -- --test-threads=1` with the chunk's
binaries deleted after each run — the same build profile and run flags as `tools/test/suite.py`
(release, offline, --test-threads=1, NF_SILENT=1, NF_ISO set), minus the 600 s per-binary wall clock
(no hang occurred; every binary finished). suite.out's last content is the ENOSPC build log, not a
total line: the "N passed, N failed" total for this round is the chunk tally above (0 failed).

## What's left

- Nothing on the task's three items. The nb-scan thread (NB_SCAN_LIVE=false, the ignored
  `drone_directly_ahead_parks_the_mover_in_0x5e` and the walkreal_07000001 re-pin note) is untouched —
  that flip belongs to its own task; walkreal_07000001_live is green here with the scan still inert.
- The nf-view tests outside the AI/walk subset (rendering, HUD, MP, menus) were not run this round
  (disk ceiling above); they were green on master at the last full-suite round and none of them read
  the changed semantics (grep: only ai_missing_states_live and stuck_guards_sweep reference
  NO_OPPONENT/has_opponent/opp_have_frame).

## Live checks for the main agent

- [S] suggested, one PCSX2 FRESH-boot check that pins the grace: on 07000001, kill Bond while a hall
  guard (skins 0x39/0x36/0x38/0x3a) has him sighted; `gdbwatch.py` on a guard's drone+0x238 and
  +0x170: expect +0x170 → 0 the tick Bond's body goes class-3-invalid (health gate), +0x238 to hold
  its last value, and the guard's state to stay in 0x58/0x5c for ~30 frames after the death before
  dropping to 0x60 (HaveOpponentSight stops re-stamping once +0x170 == 0).
- [S] the mirror: with Bond alive but hidden from a 0x5c guard, the game's guard must NEVER enter 0x60
  (the head gate can't fire — +0x170 stays the player); it walks the route in 0x5c/0x5f. A
  fieldprobe of state over 10 s of hiding would pin that too.

## Usage

Session token/credit figures are not visible to me: this worker runs in a sandboxed CLI with no
cost/usage dashboard. Context-window figures: not visible to me. The only usage-shaped numbers this
environment exposes are filesystem/process ones (disk 9.9G rootfs, ~840M free at report time).
