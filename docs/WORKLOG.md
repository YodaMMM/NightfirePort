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

## drone-block-swaps-g055-4 (started 2026-11-04 15:16 UTC)

Branch s/drone-block-swaps-g055-4 cut from origin/s/drone-block-swaps-g055-3,
origin/master (182408f88) merged clean (no conflicts). Continue g055-3's hand-off: the two
target walks — 07000016's chaser-defeat real-damage walk (the dry-ammo route-economy root;
levers: the AR switch for the clears, cover-positioned fights, the ramp-mouth ambush's five
fixed spawns killed before the descent) and 07000014's delta-exit real-damage walk (levers:
clear #281/#283 before the descent legs, or hold the door-wall lane while firing; keep the
(29.99, 0.61, 38.00) descent-landing pin). Real input only; no top-ups; shared helpers
opt-in. Done = both green + every walkreal_*_live green + the full suite total line.
Also to measure on this tree: the g055-3 merge's two shifts (07000003 real_damage,
07000011_live 2/4) — master advanced since (turret-round-spread, ring-beams-3 et al.).

## drone-block-swaps-g055-4 (progress 2026-11-04 ~18:15 UTC)

The two target walks verified green on the pushed tree, whole, serial: `walkreal_07000014_live`
9 passed / 0 failed / 1 ignored (153 s), `walkreal_07000016_live` 14 passed / 0 failed /
2 ignored (74 s). The FULL walkreal_*_live sweep ran: 80 binaries, 246 passed / 3 failed /
20 ignored; the only 3 failures are the g055-3 hand-off's master-merge shifts in other
levels' files (07000011 spawn-to-load-point real_damage + _agent, 07000003 full-route
real_damage) — measured still red on this tree, owners' files. Full-suite grind in progress
(resumable per-binary chunker, fresh state): 592/1104 binaries, 2744 passed / 11 failed /
190 ignored; failures = the 3 allowed slim-ISO disc_golden + identity_disc (needs the full
retail image, 1,191,072 sectors [D docs/build-identity.md]; the sandbox has the slim onfoot
ISO, 518,435 sectors measured) + the GPU-surface set (glass_guards_14_live 1, fix_0c_pylon_live
2, probe_pylon_render 1 — wgpu adapter NotFound headless, the last three matching the October
headless record). Report + totals to follow.

### drone-block-swaps-g055-4 outcome (2026-11-04 ~19:05 UTC)

Both target walks green and verified whole on the pushed tree (07000014 9p/0f/1i,
07000016 14p/0f/2i); the full walkreal_*_live sweep 80 binaries 246p/3f/20i (the 3 = the
g055-3 hand-off's other-level shifts, measured still red, owners'); the FULL four-crate
suite ran: 1104 binaries built, all run serially — 4202 passed / 15 failed / 339 ignored
(4203 effective with the second crate's fix_ai_remainder_live copy, both copies green
directly). Failures classified in docs/reports/drone-block-swaps-g055-4.md: the 3 allowed
disc_golden, identity_disc (needs the full retail image), 5 GPU-surface tests (wgpu
adapter NotFound headless), the 3 pre-existing walkreal shifts, 2 mp timeouts. Report:
docs/reports/drone-block-swaps-g055-4.md.

## drone-block-swaps-g055-5
Started 2026-11-04 20:24 UTC. Branch s/drone-block-swaps-g055-5 = origin/s/drone-block-swaps-g055-4
(afd342e60) merged with origin/master (2b02ff2ec lineage). Task: the last three walks before G-055 —
walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage,
walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage (panic walker_b2.rs:170)
and ..._real_damage_agent (panic walker_b2.rs:641) are red on the merged tree, green on master. For
each: say what changed for the guards on that level, then re-route or re-pin with the opt-in levers
the 14/16 walks used. FirstAttack/HandleImpact read correct — do not change them to make a walk pass.
Every walkreal binary + the full suite total line.
- (20:24) branch cut, master merged clean, WORKLOG started.
