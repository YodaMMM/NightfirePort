# WORKLOG

## drone-block-swaps-g055-2 (started 2026-11-03 01:21 UTC)

Task: land the G-055 drone block swaps without the six regressions. First merge
origin/s/drone-block-swaps-g055 (FirstAttack's alt-state branch [M 0x148010:100-131],
the second-switch flush, the tase fall on 0x090a) into this branch cut from master;
on master that branch turns six tests red:
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage
- walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it
- walkreal_07000016_live::the_chasers_punch_lands_the_record_damage
- walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage
- rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway
- abseil_elevator_live::the_elevator_jumper_opens_the_door_and_attacks
Do: (1) for each red find which swap fires and whether the game would fire it there
(the +0x5A4 / +0xC48 == 100 / +0x4F8 gates, each drone's real values from level data
[D]); a swap firing where the game's gates would not is a port bug — fix with
citations. (2) Where the game really swaps, the test's expectation or the walk must
follow the game (cite [M]); keep [L] pins unless proven wrong. (3) All six green plus
drone_block_swaps_live; run every walkreal_*_live, every rook/abseil/elevator test,
then the full suite.

### drone-block-swaps-g055-2 — 2026-11-03 (sessions 1-2)

The G-055 re-land (revert-of-revert of c0450ef6) plus the six reds triaged: rook06 and
abseil_elevator were the tests encoding the pre-g055 model — the game's FirstAttack tail
[M 0x148010:100-131] really flushes the ch-16 guard (row-2 alt 0x2a, FirstAlertState 0 on 06
[M 0x147f58]) and the jumper's row-7 chain 0x85→0x89→0x63; both tests now follow the game and
run green. The 07000014 tripbomb walk went green with the trench-guard kill and the stale-gi
re-finds; the 07000016 punch walk with the gated shelter-door counter-fire. Remaining: the
07000014 real-damage west-route swarm and the 07000016 real-damage defeat walk (report:
docs/reports/drone-block-swaps-g055-2.md).

## drone-block-swaps-g055-3 (started 2026-11-04 10:14 UTC)

Branch s/drone-block-swaps-g055-3 from origin/s/drone-block-swaps-g055-2 (82128666),
origin/master (66822831) merged in (the one conflict — the stairwell-foot y cap in
walkreal_07000014_live.rs, two copies — resolved to the looser 1.1 with both sides'
justifications). Task: the last two real-health walks under the G-055 hunt — 07000014's
delta-exit walk (dies ~14137 at (-48,30) to the p5=100 hunters #281/#283) and
07000016's chaser-defeat walk (dies ~9241 on the pit floor to the chaser + pit squad).
Real input only; no health tops; shared helpers stay opt-in.
- [11:00 UTC] 07000016 iteration chain (all measured): the kill-loop side-fire on the pit
  soldiers (she is skipped while inside her 2.0 punch band), the hand-back stand (breaks at
  +0 — the soldiers are not exposed from the hand-back spot), the proactive pulse-1 sit
  (no exposure through the shelter walls), lane_fights+back_off (WORSE — the stand traded
  61.6 in one fight, died on the route at 1926; reverted), the opt-in finish_fights
  escalation in the shared helper (flag default false; only the d1 real-damage walk sets
  it): the un-dropped hunter now eats one determined shoot_bounded pass — fights kill true
  (4 kills in the last trace) — and the leg-wise clear_hostiles_16 with corpse walks. The
  run still dies (last: 10684 at the ramp foot — the ambush crossfire during each fight
  costs 3-39 health a kill; Bond's own dps with the carried guns is the bottleneck). The
  next session's levers: a higher-dps carried gun for the clears (the guards' own ARs via
  an early switch), cover-positioned fights, and killing the ambush's 5 guards BEFORE the
  ramp (their spawn/hold spots are fixed [D] — a pre-positioned stand at the ramp mouth).
- [11:47 UTC] Report written (docs/reports/drone-block-swaps-g055-3.md): both target walks
  not green yet (07000016 diagnosed to the route-economy/dry-ammo root with the measured
  attempt chain and next levers; 07000014 not reached), the four earlier fixes verified
  green on the merged sim (nf-game 1051/0/8 whole; 07000014 binary 8/1, 07000016 binary
  13/1 — only the target walks red), the two master-merge shifts (07000003/07000011)
  documented for their owners, the critical-binary sweep results recorded. finish.sh
  with `blocked` + this reason follows.
