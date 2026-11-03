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
