
## level-12-engage-range
Started 2026-11-05 11:08 UTC. Branch s/level-12-engage-range (cut from origin/master f68a72216).
Task: s/level-12-invisible-wall-spots found that after the real alarm on 07000012, drone #6
holds at the corridor mouth with clear sight at 9-10 u and #7 stays in cover, so nobody
reaches the cafeteria. Compare `engage_range` for skins 0x5000008 and 0x5000006 with the
game's Drone+0x1A0 [M] and the combat_state advance rule. If the port holds where the game
advances, fix it. Then test the no-sight advance against the corner at (22.47,36.64): does it
stop short of the game's arrival radius? Run the 12 walks and report them.

Progress 2026-11-05 11:08 UTC: branch cut, WORKLOG started. Sandbox survived (repo+work+ISO
present, target/ 5.2G warm from the previous session; closed out level-1b-platform-touch's
ntfy-failed report section on that branch first).

Progress 2026-11-05 11:10-12:15 UTC: the comparison, [M-cited]:
- engage_range VALUES match. The game's sight-class switch `NDrone2_DefaultInit` 0x14b300
  (switch(piVar19[0x1b]), the LAB_0014b8a4 region) case 2: +0xF0 = FRand(3.0)+12.0 = 12-15,
  +0xE8 = 16.0, +0xFC = +0xF0+1.0+FRand(3.0); cases 1/3/4/5 and the default match the port's
  table (drone.rs:1985-1988, 2017) value-for-value; the only per-level overrides are for
  0x7000011/0x7000014 — none for 0x7000012. Both skins (plc #5 = 0x5000006, #6 = 0x5000008)
  carry param 14 = 0x2 → the same 12-15 band.
- The advance RULE matches. `DroneFunc_CombatState` 0x148338: COMBAT (case 0) returns 0x5a iff
  +0xF0 <= +0x1A0; COMBAT_OUT_OF_RANGE (case 2) returns 0x58 when +0x1A0 < +0xF0; the fire
  states (cases 0x10/0x18) advance iff routed (MoveToObject result < 4 or == 10) && +0xF0 <=
  +0x1A0; the MoveToObject radius arg is min(engage, 6.0) (0x40c00000). The port
  (drone.rs:10723-10742, 10759, 10787) implements the same arms; +0x1A0 = the 3D distance
  opponent-obj+0x30 to own-obj+0x30 [M Drone_GetOpponentInfo 0x138f68] = the port's self.dist
  (drone.rs:11361). So the port does NOT hold where the game advances: at dist 9-10 with
  engage 12-15 BOTH hold in the fire states, and both advance only when dist >= engage.
- The FOUND difference (goal-move arrival radius): the game's MoveToGoalPosition arrives when
  the route distance drops under route+0x8C = Drone+0x8EC [M 0x1517d8:49-52], which
  `SetupRouteToPosition` writes from the goal's radius param on every position route build
  [M 0x158b10 `*(float*)(puVar9+0x46) = param_1`; 0 → 2.0], `DefaultInit` seeds 2.0
  [M 0x14b300:579], and `SetupRouteToObject` never rewrites [M 0x158948 — no route+0x8C store].
  Per-caller radii: the alarm-point goal = the AIPoint's own +0x14 accuracy [M 0x153008:20-23,
  45-48], FindRunToPoint the same [M 0x153168:39-43], sound/drone alerts 2.0 [M 0x154aa8/0x154b20],
  FindCover 0.5/1.0 by corner class [M 0x152548:174-181], ambush cover 0.5 [M 0x152958:207-208],
  safety 1.0 [M 0x152250:126-127], mission fresh 1.0 / retry 2.0 [M 0x153a30:141-142, 168-169],
  the closer-position probe 3.0 [M 0x155e10:30], interrogate assist 10.0 [M 0x15add0]. The
  port arrives at a hardcoded 2.0 everywhere (drone.rs:5852) — the AIPoint accuracy field is
  even parsed (aipoint.rs `accuracy`) but unused by the goal paths.
- Fix attempted and REVERTED. Two forms (goal-path radii only; + the persistent route+0x8EC
  carry into MoveToObject) — BOTH turned walkreal_07000001_live's two real-damage walks red
  (Bond dies; walker.rs:279 [real-damage] panic): the true radii bring guards closer to their
  goals (mission 1.0, alarm accuracies < 2.0) than the walks' damage balance, which was tuned
  on the 2.0 stand-in. Baseline proof: stashed patch → the 001 test passes (4.6 s), unstashed
  → fails. Left out of the tree per the merge rules; the exact patch is described here and in
  the report for the next session, together with the cross-level walk rebalance it needs.
- The corner question (22.47,36.64): the game's no-sight arrival radius is the GOAL's radius —
  for the 07000012 alarm response the sound/drone-alert goals carry 2.0 [M 0x154aa8/0x154b20],
  so the port's 2.0 does NOT stop short of the game's arrival radius for those; only goals
  whose radius ≠ 2.0 (AI-point accuracies, mission 1.0, interrogate 10.0) diverge today. A
  live check (below) pins the real game's stop distance at the corner.
- Walks + suites on the final (reverted = master-behavior) tree: the 12 level walks
  walkreal_0700000{01,02,03,04,05,06,07,08,09,0a,0b,0c}_live = 89 passed / 0 failed / 5
  ignored; walkreal_07000012_live 3/0, corridor_guards_07000012_live 1/0,
  enemies_07000012_live 3/0; nf-game full suite 1077 passed / 0 failed / 8 ignored;
  nf-formats+nf-audio green except the 3 expected disc_golden slim-ISO failures.

## guard-goal-arrival-radii
Started 2026-11-05 18:12 UTC. Branch s/guard-goal-arrival-radii (master 2f0c88e2c + origin/s/level-12-engage-range
merged). Task: guards arrive under each goal's own radius — route+0x8C = Drone+0x8EC, seeded 2.0 [M 0x14b300:579],
rewritten by SetupRouteToPosition from the goal's radius (0 -> 2.0) [M 0x158b10], NOT touched by SetupRouteToObject
[M 0x158948]; MoveToGoalPosition arrives under it [M 0x1517d8:49-52]. Port uses flat 2.0 (drone.rs move_to_goal +
move_to_object). Census [M]: alarm point + run-to point = AiPoint +0x14 accuracy; sound/drone alert 2.0; cover 0.5/1.0;
ambush 0.5; safety 1.0; mission fresh 1.0 / retry 2.0; closer-probe 3.0; interrogate 10.0. level-12's fix regressed the
001 real-damage walks (Bond dies) and was reverted — form 1 first, walks re-routed per-level opt-in, never weaken.
