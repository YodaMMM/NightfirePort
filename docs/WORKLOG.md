# WORKLOG

## guard-mode-c4-glass (branch s/guard-mode-c4-glass)

Started 2026-11-05 ~19:06 UTC. Task: two player-visible divergences.
1. Drone::attack_mode (+0xc4) never set -> abseil landing (AbseilSlide 0x169730:36,
   StepOff 0x1738e0:21) and the mode-7 arm near "set_2nd_behaviour" drop the drone
   to mode 0. Game: NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:22-23 writes
   +0xc4 = GetDTYPENEW(drone, +0xc7, 0), runs from NDrone2_DoModeSettingsNEW 0x14c600.
   Port the write at init, fix the comment, prove with a live walk (landed abseiler
   on 07000011 fights with the right mode).
2. glass_pass: false at guard spawn sites in drone.rs is a shortcut. Game clears the
   stop for every record without bit 0x800 [M Bullet_update 0x12e7b0 disasm
   0x12f030-0x12f058]. Arm it from the record as the player side does, then re-pin
   real-damage walks that relied on the old behaviour (08 station route first).
   No draws removed or reordered.

Progress log:
- Sandbox was reset; full setup redone (clone, work-slim, ISO, rust, cargo fetch).
- Branch s/guard-mode-c4-glass created from origin/master (2f0c88e2c) and pushed.
- Fix 1 (+0xc4 attack_mode): type_setup now writes `attack_mode = dtype_of(d, dtype, 0)`
  [M 0x14cbd8:22-23, game order before the mode byte] + the kind-0x18 astronaut rewrite
  0x1d [M 0x14b300:725]; field comment corrected (the "no writer" note was wrong — OLD
  chain tail-calls NEW 0x14cfa0:16). Disc abseilers (07000011 x2, 0700004a x3) are dtype 3
  with param-18 bit 16 clear -> attack_mode 0 (unchanged landed mode, now derived not
  hardcoded). Test attack_mode_c4_live: ambush-bit guard expects 0x10, astronaut 0x1d,
  disc abseiler 0 — verified RED with the write reverted, GREEN after.
- Fix 2 (glass_pass): all five guard spawn sites (melee 0x1789c0, fire_weapon 0x178f68,
  copter 0x17d828/0x17d9f0, GT turret 0x180498, space laser record 107) now arm
  `glass_pass` from the weapon record +0x6c bit 0x800 [M Bullet_update 0x12e7b0 disasm
  0x12f048-0x12f058] — the player-side law. 08 real-damage walk re-pin in progress.
- 08 walkreal re-pin (partial): added the "stairwell mouth" fight stand at difficulty >= 2
  and extended the "lower level east" platform-mouth stand gate to >= 2. Verified on the
  real-damage trio: Operative (d1) + the plain walks pass (4 passed), the Agent (d2) and
  00 Agent (d3) runs STILL FAIL — the d3 death moved from tick 25209 to 27933 (a later,
  different spot); at the mouth stand #19 held "out of sight" through 6 stands (sight
  blocked from the foot) and the stand burned ~1000 ticks for 3 kills (#20/#23/#24).
  The two Agent variants need the next session (see the report's what's left).
- Regression spot-checks on the type_setup change: nf-game lib 679 passed / 5 ignored,
  abseil_death_live 5 passed, stairwell_guards_07000011_live 3 passed,
  abseil_elevator_live 3 passed. The FULL suite did not fit (walkreal binaries alone are
  minutes each; the sandbox deadline hit) — the per-binary list and the remaining runs are
  in the report.

## guard-glass-finish (branch s/guard-glass-finish)

Started 2026-11-06 ~00:16 UTC. Task: merge origin/master into origin/s/guard-mode-c4-glass,
push as s/guard-glass-finish; fix the two Agent walks of walkreal_07000008_live test-side;
do not loosen a [L] pin; run the binary whole and every walkreal possible.

Progress log:
- Branch created from origin/s/guard-mode-c4-glass (4e4bcd668) + merged origin/master
  (576909ed6, 11 commits) -> merge commit ce2ee9489, pushed. No conflicts.
- Read docs/reports/guard-mode-c4-glass.md (the two fixes are code-done; the 08 Agent
  re-pin was the leftover).
- Measured the merged-master tree (the timeline moved vs the pre-merge report): d2 died
  tick 25036 at the stairwell foot, burst pair #19/#20 point-blank (d 0.6-0.7, full hp);
  d3 died tick 25116 on the west ledge, same pair. NF_RF_TRACE/NF_RF_GEOM ground truth:
  (a) the mouth stands ran eleven 61-tick stands on #19 with aim None at every one — the
  foot's sight into the platform is blocked by the mouth's SOLID wall band (the eye ray
  stops at x~105.25 inside the owner-22 runs; NOT glass — the glass-pass sight trace clears
  nothing there), while the pair walked 8.5 u to point-blank; (b) the stairwell-top pair
  #23/#24 shot into the fighting=true window; (c) a failed stand cooled #20 and the pick
  bailed the whole response; (d) the next stand fought the zero-damage #23 at 11 u while
  #20 (59.0 in 7 hits, 4.4 u, visible) chewed health 68 -> 0.
- Fix 1 (08-local, d2/d3-gated): fight_close_guards gained a VISIBLE-only sister
  (fight_close_guards_visible) — the pick keeps only close guards a sight ray reaches; a
  buried nearest guard opens no stand. The "stairwell mouth" foot stand became visible-only;
  a NEW "stairwell top" stand clears #23/#24 before the climb; the "platform mouth" stand
  (already d2/d3-gated) now fights the pair from the platform where it is visible (the d2
  run killed #19 in 57 ticks, #20 in 223). The plain sister's call sites pass false — the
  Operative/Agent-pinned timelines stay byte-exact.
- Fix 2 (opt-in in the shared helper, lesson 7): DamageLog gained `hurting_first` + the
  wrapper with_real_damage_glass_fights (glass_sight + hurting_first + head-hunting), set
  only by the 08 d2/d3 tests. return_fire under hurting_first: the pick orders the VISIBLE
  shooters by the damage they dealt over the 240-frame window (the lane pick's order, the
  plain windows), cooled candidates are dropped instead of bailing the response, and a
  cooldown bends to active lethality (a visible guard dealing > 15 in the window is
  switchable). stand_and_fire gained a mid-fight RETARGET under hurting_first: every 30
  ticks the fight switches to the worst visible off-cooldown shooter when his 240-frame
  damage beats the current target's by > 3 (the d3 death's fight#23 opened on the only
  candidate at 24850; the burst pair opened ten ticks later). Also exposed_impl gained the
  opt-in GLASS_SIGHT sight rule (the sight ray passes intact glass like the game's own round
  flights do [M Collide_Update 0x1ea398:216-229; L weapon_data every gun bit 0x800 clear]) —
  kept for the through-pane fights even though the mouth blocker proved solid; and
  stand_and_fire gained the env-gated (NF_RF_GEOM) geom_dump probe on the first blind tick,
  mirroring return_fire's existing diagnostic.
- Result: d2 walk_the_whole_mission_on_real_input_real_damage_agent GREEN (whole binary:
  5 passed / 1 failed). d3 reaches the bottom terminal's Hind fight (from tick 25116 to
  27552-29996 across variants) and dies on the documented launch self-blast — the round
  detonates on the cabin's own frame band next to Bond: explosion at (113.6, 14.4, -177.6),
  d 3.4, exactly the pane-8 frame, the round bearing ~0.13 rad off the gated corridor (the
  -26 world hit at tick 27552). Four gate variants tried and measured (cone yaw+/-0.15,
  cone yaw+/-0.30 x pitch+/-0.15 — starves the fight, 0 launches; a |cross|<0.05 cursor
  gate — starves it, the Hind's own blasts chip Bond down, death 29996; a 20-tick cursor
  stillness gate — opened t43, the round still flew 0.13 rad off, death 27559). The
  spawn-tick bearing is not recoverable from the gate-tick cursor state; the fix needs the
  live checks below (or a game-side look at the AT-420 spawn bearing law). Retired gates
  removed; the fight ships with the previous session's gates + the retarget machinery.
- Whole-binary runs (this branch, quoted): walkreal_07000008_live 5 passed / 1 failed
  (the d3 00agent Hind-fight self-blast above); walkreal_07000005_live 14 passed;
  walkreal_07000011_live 3 passed / 1 ignored; walkreal_07000012_live 3 passed;
  walkreal_07000013_live 2 passed / 0 failed / 1 ignored (the warm-chain real_damage test
  carries the pre-existing #[ignore] at 07000013_live.rs:405 — one earlier loop run of a
  mid-build binary printed FAILED with death tick 1831 + other 38.0; it did not reproduce
  on the final build and the binary there was stale mid -j4 build — see the report).
- The full suite did not fit: `cargo test -p nf-view --no-run` alone (970 test binaries)
  exceeded the 10-minute tool cap; the walkreal set above ran from the binaries it had
  finished. No src/ changes were made (nf-game/nf-formats/nf-audio untouched — the lib's
  own suites are unchanged from the merge state).
- ntfy failed (see the report).
