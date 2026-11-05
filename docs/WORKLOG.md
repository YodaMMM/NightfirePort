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
