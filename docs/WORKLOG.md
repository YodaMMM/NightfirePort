# WORKLOG

## 987-mission-stats-carry

- Task: the pause SCORE tab counters must survive a mission's level change (07000005 → 07000006).
- Branch `s/987-mission-stats-carry` created from `origin/master` (26543436).
- Plan: (1) find in the decompile what PlrMissionStats holds and who zeroes it (mission start vs
  every level load: ResetMap_Load, Player_Init, the PlrStat_* inits) and what the SCORE tab's
  C_GCPAUSE_Handler 0x204408:447-498 reads; (2) make the port carry exactly that across the level
  change and reset exactly where the game resets; (3) real-path test: earn a Dispatched and a Bond
  Move on 07000005, cross into 07000006 through the real level exit, counters hold; sister:
  mission restart zeroes them if the game does.
- Note: the driver's task tail says "Branch: s/mission-stats-carry" but the header, task id and
  finish command all say s/987-mission-stats-carry; using s/987-mission-stats-carry.
- (12:30-12:55 UTC) Research (two subagents): the game zeroes PlrMissionStats (0x2bcc20, 0x1C0) +
  BondMoments (0x2bcde0) ONLY via `PlrStat_ResetForMission` 0x1993a0 — called by `Mission_Init`
  0x182270:65 gated on the loaded map BEING the mission's base map (MissionData 0x2a4350: The
  Exchange 05/06/07/08 = order 0..3, base 05), by `PlrStat_Init` 0x198318 (cold boot) and by
  `Menu_MapDefaultCodename` 0x1f7fa8:60 (mission select). The level-exit path
  (`Trigger_Activate` 0x1b2318:108-116) `Player_RamSave`s the tally (PlrMissionStats[0] 0x38 +
  BondMoments 0x14 into the RamSave staging buffer) before `ResetMap_LevelToLoad(next,1,0)`, and
  the warm load's `Player_RamLoad` 0x1957a8:15-16 copies both back; `Player_Init` 0x1910f8:187
  then re-stamps the timer base (`PlarStat_LogTimerUnpause`). The pause SCORE tab reads the live
  record via `PlrStat_GetScore(1)` (C_GCPAUSE_Handler 0x204408:413 → 0x1984b0:66-84, the 0x07-map
  rows refill from PlrMissionStats[0] per call). Doc corrections found: player-stats.md's two
  GetScore branches are swapped (0x09 → NewScoresRef, 0x07 → PlrMissionStats) and the pause tab
  calls GetScore(1), not GetScore(0).
- (12:35-13:00 UTC) The port's reset gate was already right (drone.rs's part-0 latch never fires
  on the hop); the loss was the scene swap: the warm `RamSave` omitted the tally (the gap
  player.rs documented). FIX: `RamSave::tally: Option<TallySave>` (+ `with_tally`, the
  `Player_RamSave` 0x195670:10-11 shape) carried by `warm_exit_save` (the
  `start_async_load` campaign arm + the weapon-assets reload), `entry_checkpoint_save` (the
  game's post-load re-save 0x1a7ef8:325 reads the resident record — pre-latch it comes from the
  carry), seeded into `Drones::tally_carry` at load and restored at the `Mission_Init` latch
  AFTER the part-0 wipe (the game's order) with the `log_timer_unpause` re-stamp
  (`Progress::restore_tally`). The pause-restart cold arm (save = None) still zeroes via the
  part-0 gate; the fail-restart serve restores the entry tally (0x216618:21).
- (13:00-13:20 UTC) crates/nf-view/tests/mission_stats_carry_live.rs — the real-path pair. Main:
  walker on 07000005 → teleport+fire kill (Dispatched 1) → porch plate ch 0x5C → crouched truck
  ride → bond move 101 (BondMoments[0]=1) → the castle-door exit latches (0x07000006) →
  `warm_exit_save` + `load_with_carry` + the first tick → the counters hold, a kill on 07000006
  stacks (dispatched 2), the entry checkpoint and the fail-restart serve carry them. Sister: the
  pause-restart cold arm (load_with_carry(None)) → the part-0 wipe → zeroed, spawns re-logged.
  Both pass in 11.7 s. Test plumbing found en route: the post-award window is input-disabled
  (kills don't land), the walk must start from the level start (the kill spot's nav hits a gate-0
  exit trigger), and the elevated first guards need the pitch-aimed shot.
- Baseline suite (pre-change tree, started 12:27): 3264 passed, 11 failed, 200 ignored — the
  failures are the known slim-ISO set (3 disc_golden + identity_disc), the pre-existing
  fix_0c_pylon_live render check and an mp_online_failures OOM crash. Final-tree suite restarted
  13:20; total to be reported.
