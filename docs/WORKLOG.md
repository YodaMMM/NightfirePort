# WORKLOG

## stuck-guard-combat-sweep

- Task: extend the stuck-guard sweep to combat movement and the 07000005 helipad.
- Branch `s/stuck-guard-combat-sweep` created from `origin/master` (26543436).
- Plan: (1) add the combat movers to the sweep's moving set (cover seek / run to cover,
  `DroneMove_NoBunching` waits, `DroneAnim_SetCombatMoveAnim` 0x13cfb0 anims, strafe/approach);
  (2) on 07000005 walk Bond into the helipad courtyard with real input, engage guards, log guards
  stuck in a moving combat state; (3) fix the class in shared code with [M] cites, sister tests.
- (10:44-10:52 UTC) Setup verified (repo, work/, ISO, nf-queue worktree all present; alsa-prefix
  needed PKG_CONFIG_PATH re-export). Branch pushed with the WORKLOG entry. Suite plumbing: the
  first background start died on `cargo` not on PATH, the second on ENOSPC (9.9 GB disk full —
  the dead-pane session's 5.6 GB target tree). Freed 1.0 GB by deduping the deps dir's duplicate
  rlibs (newest kept per basename), 160 MB examples/binaries, 467 MB registry/src (refetchable);
  4.6 GB free before the rebuild.
- (10:55-11:10 UTC) Subtask 1: the sweep's moving set extended with the combat movers
  (crates/nf-view/tests/guard_stuck_sweep_live.rs): COMBAT_OUT_OF_RANGE 0x5a (the approach chase
  [M 0x164690:38-48]), COMBAT_WAIT 0x5e (`DroneMove_NoBunching`'s parked mover [M 0x13cfb0:35-39]),
  AIM_BACKOFF 0x6d [M 0x1501a0], the strafe family 0x73-0x7a [M 0x150c18], the scare cover-seek
  0x86/0x87 [M 0x168aa8/0x168d18], UNDER_COVER_RETURN 0x99 [M 0x16a9c0 case 3]; plus the
  combat-band anim check (a guard in 0x56/0x58-0x5f/0x67-0x71/0x73-0x7a playing a
  `DroneAnim_SetCombatMoveAnim` move anim counts as wanting travel). HIT rows now carry the anim id.
- (11:05-11:27 UTC) Subtask 2: crates/nf-view/tests/stuck_guard_combat_05_live.rs — the 07000005
  real-input walk. Run 1 (the single nav legs): Bond reached the road head (30.12, 36.21, 66.31),
  fell off the road edge at (48, 34, 55.66) (floor_below 28.93), wedged below the gate plateau at
  (34.64, 34.72, 49.58); the alarm engagement then froze EIGHT guards in STEP_AIM_LEFT/RIGHT
  (0x73/0x74, anims 0x5a/0x5b) inching < 0.1 per 3 s for the whole 60 s window, plus drone#6
  RUN_FOR_COVER frozen 13.53 from its goal. The pad courtyard itself NOT reached (the descent
  from the y-34 ledge to Bond's target at y 35.2 blocks the walker — the walkreal wedge).
- (11:27-11:35 UTC) The step-wall class diagnosed: the combat-move CHOICE probe (`can_move`,
  knee/chest rays at +0.5/+1.4) passed where the MOVER (`move_by`, feet/knee rays at +0.0/+0.5
  plus the 0.4 overshoot) rejected — every completed step anim re-picked the same blocked step
  (0x73/0x74 ↔ 0x68 forever). The game cannot diverge this way: its choice probe (CanStepLeft
  0x14f640 family) and its mover share the one MoveTest + GetBoundsPushVectorForSphere 0x157bd0
  machinery [M]. FIX: `probe_move` (move_by's test as a dry run) now gates the choice's pair(),
  the evasive dodge probe and the strafe UPDATE's blocked exit [S the ray set itself; the
  agreement is the port's requirement].
- (11:35-11:50 UTC) Run 2 (the waypointed road walk + the fix): the step wall GONE; the run
  surfaced the NEXT class — SEVEN guards parked in COMBAT_NO_SIGHT 0x5c (RUN_AIM anim 0x08
  playing) at (28.15, 37.00, 48.06-48.10), unseen_ticks 18,000-27,000 (5-7.5 min), route target =
  Bond's feet 1.8 u BELOW their ledge, route_blocked false. That is the sight-loss exit the port
  has disabled (`if false &&` at drone.rs:9866 — the combat-sight-loss-exit task's territory, the
  +0x238 30-frame hold [M 0x148338 / 0x14ad58]): NOT fixed this round, the MECH dumps are in the
  report. Sister `no_guard_loops_a_blocked_combat_step` added (the class invariant); it passes
  post-fix on every flow run and ALSO passes pre-fix on the three repro flows tried (the freeze
  is flow/RNG-dependent — run 1's exact chaos froze 8 guards; the alarm-only and waypointed
  flows don't) — documented as the invariant guard, not a proven repro.
- (11:35-11:50 UTC) Blast radius on the fixed tree: stuck_guards_sweep (nf-game) ok 16.8 s,
  guard_move_checks_live ok 4.1 s, walkreal_07000005_live 6 passed / 2 ignored, the extended
  guard_stuck_sweep_live 0 hits across all 17 on-foot levels (the alarm path), the 05 pair
  (measurement + sister) 2 passed.
