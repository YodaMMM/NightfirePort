# 987-mission-stats-carry — the pause SCORE counters survive the mission's level change

Branch `s/987-mission-stats-carry` (from `origin/master` 26543436; master did not move during the
session). Note: the driver's task tail says "Branch: s/mission-stats-carry" but the header, the task
id and the finish command all say `s/987-mission-stats-carry`; this branch is the one used.

## What the game does (all [M] unless noted)

- The pause SCORE tab (`C_GCPAUSE_Handler` 0x204408:413 → the page-3 build at :418-498) reads the
  ScoringTable row values through `PlrStat_GetScore(1)`; for the 0x07 maps GetScore refills the rows
  from `PlrMissionStats[0]` on EVERY call (0x1984b0:66-84), so the tab shows the live tally. (The
  param is 1 — the no-commit read; docs/fields/player-stats.md said GetScore(0) and had the two
  GetScore branches swapped — both corrected in the doc this round, with the addresses.)
- The tally is zeroed ONLY by `PlrStat_ResetForMission` 0x1993a0 (memset PlrMissionStats 0x1C0 +
  BondMoments 0x14): `Mission_Init` 0x182270:65 calls it **only when the loaded map IS the mission's
  base map** (MissionData 0x2a4350: The Exchange = 07000005 order 0 / 07000006 order 1 / 07 / 08,
  base 05), plus `PlrStat_Init` 0x198318 (cold boot) and `Menu_MapDefaultCodename` 0x1f7fa8:60
  (front-end mission select). `ResetMap_Load` 0x1d6360, `ResetMap_GameInit` 0x1d5080 and
  `Player_Init` 0x1910f8 touch nothing in the block.
- The level change carries the tally twice over: `Trigger_Activate` 0x1b2318:108-116 calls
  `Player_RamSave` 0x195670:10-11 (which copies PlrMissionStats[0] 0x38 → RamSave+0 and BondMoments
  0x14 → RamSave+0xA04) before `ResetMap_LevelToLoad(next,1,0)`, and the warm load's
  `Player_RamLoad` 0x1957a8:15-16 copies both back (whole body gated on `IsWarmReset != 0`).
  `Player_Init` 0x1910f8:187 then re-stamps the timer base (`PlarStat_LogTimerUnpause` 0x199550:
  +0x2C := now, +0x34 := 0).
- A pause-menu RESTART loads the base map COLD (0x204408:139-155 → `ResetMap_LevelToLoad(
  BaseMapHCode, 0, 1)`) → `Mission_Init` on the base map wipes the tally. A fail-screen RESTART
  LEVEL restores the level-entry snapshot (`P_ENDMISSION_Handler` 0x216618:19-21 → `Player_RamLoad(…,1)`),
  so the mission's earlier counters survive it.

## What the port did wrong, and the fix

The port's `Mission_Init` latch already had the right part-0 gate (it never fires on the 05→06 hop).
The loss was one layer up: the level change builds a fresh Sim whose `Progress::default()` zeroes the
tally, and the warm `RamSave` omitted it (the gap `player.rs` itself documented: "the game also saves
PlrMissionStats (0x38), SSysItems (0x28) and BondMoments (0x14) — none of which the port has").

Fix (the game's exact shape):
- `RamSave::tally: Option<TallySave>` (`player.rs`) — the `Player_RamSave` 0x195670:10-11 pair
  (slot-0 record + BondMoments), attached by `RamSave::with_tally`.
- `nf_view::warm_exit_save(sim)` — the capture `start_async_load`'s campaign arm and the
  weapon-assets warm reload hand the worker (player state + mission globals + tally).
- `entry_checkpoint_save` overlays the tally from `Drones::tally_carry` pre-latch (the game's
  post-load re-save 0x1a7ef8:325 copies the RESIDENT record, which on a warm load already holds the
  RamLoad restore because `Player_RamLoad` runs inside `Player_InitWeapon` before the re-save).
- `load_impl` seeds `Drones::tally_carry` from the save; the `Mission_Init` latch (drone.rs) restores
  it AFTER the part-0 wipe — the game's order (Mission_Init zeroes on a base-map load,
  `Player_InitWeapon`'s RamLoad puts the snapshot back) — via `Progress::restore_tally(tally, now)`,
  which applies `Player_Init` 0x1910f8:187's `log_timer_unpause` re-stamp on the port's frame clock
  (without it the carried +0x2C stamp would wrap the elapsed accumulator).
- The pause-restart cold arm (save = None) still zeroes via the part-0 gate — now asserted by test.
- The fail-restart serve (`fail_restart_save` from the entry checkpoint) carries the entry tally —
  the game's restart-level behaviour.

## Verification

- New real-path pair `crates/nf-view/tests/mission_stats_carry_live.rs` (NF_ISO-gated), both pass
  (11.7 s): 
  - **the_tally_survives_the_exchange_hop**: the walker drives Bond with the real input only — a
    teleport+fire kill through `step_bond` (Dispatched 1), the gatehouse porch plate (ch 0x5C), the
    crouched truck ride (bond move 101, BondMoments[0]=1), the castle-door exit latches
    (level_exit = 0x07000006); then the App's exact crossing functions (`warm_exit_save` →
    `load_with_carry` → the first `step_bond` tick, whose latch applies the restore): the counters
    hold on 07000006's live record (what the pause SCORE tab draws from — the tab's source sync is
    `pause_stats`, pinned in-crate), a kill on 07000006 stacks (dispatched 2 — logging continues on
    the carried record), the entry checkpoint and the fail-restart serve carry them, the timer
    re-stamp ran (+0x34 = 0, elapsed monotonic).
  - **a_mission_restart_zeroes_the_tally** (sister): the pause-restart cold arm
    (`load_with_carry(None)`) → no carry, the part-0 `Mission_Init` wipe → dispatched/bond moments
    zero, spawns re-logged.
- Blast radius (all pass after the change): nf-game lib + stuck_guards_sweep; nf-view
  level_carry_live, async_level_load_live, fix_scores_handoff_live, fix_death_retry_boost_live,
  fix_progress_live, bond_moves_live, models_characters_live, fix_death_restart_live,
  pause_results_flows_live, fix_restart_target_live, mayhew_end_transition_live.
- Full suite on the baseline tree (started at session start): 3264 passed, 11 failed, 200 ignored —
  the failures are the pre-existing set (3 disc_golden + identity_disc on the slim ISO, the
  fix_0c_pylon_live render check, an mp_online_failures OOM crash), none touched by this task.
- Full suite on the FINAL tree (fast runner, `tools/test/suite.py`): total line below.

### Final-tree suite total

(pending — filled in before the finish step)

## Doc corrections (the decompile settles them; no test was weakened)

- `docs/fields/player-stats.md`: the two `GetScore` branches were swapped (the disasm's flag store
  order at 0x1984b0:38-43 settles it: 0x09 maps → NewScoresRef, 0x07 maps → PlrMissionStats); the
  pause tab calls `GetScore(1)` (0x204408:413), not GetScore(0); the "whole scoring system is
  unported" port-status paragraph was stale (progress.rs ports the layer).

## What's left

- Nothing known for the counters. Port-only gaps, NOT this task's symptom, noted for follow-up: the
  pause SCORE tab's shots/accuracy/time rows still draw defaults (menu.rs:9493) — the game fills
  them from the same record; `SSysItems` (0x28) still rides the modeled inventory fields only, not a
  block copy.

## Live checks for the main agent (all optional — the decompile evidence is the load-bearing part)

1. `gdbwatch 0x2bcc20 0x38 write --seconds 30` across a real 07000005→07000006 transition: the only
   writers expected after the exit are `Player_RamSave` 0x195670 (the snapshot) and `Player_RamLoad`
   0x1957a8 (the restore); no `PlrStat_ResetForMission` 0x1993a0 write in between.
2. Pause SCORE tab on 07000006 after one dispatched + one bond move on 07000005: Dispatched 1, Bond
   Moves 1 (the fieldprobe screenshot pair, before/after the door).

## Usage

Session token/context/cost figures: not visible to me. (The two research subagents this session
spawned reported their own token totals in their final messages — 159,100 and 192,797 total — but
those are their sessions, not this one's.)
