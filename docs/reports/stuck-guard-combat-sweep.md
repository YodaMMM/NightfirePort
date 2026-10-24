# stuck-guard-combat-sweep — the combat movers joined the sweep; the step-loop class fixed; the sight-loss wall measured

Branch `s/stuck-guard-combat-sweep` (from `origin/master` 26543436).

## What's done

1. **The combat movers are in the sweep's moving set** (`crates/nf-view/tests/guard_stuck_sweep_live.rs`):
   - COMBAT_OUT_OF_RANGE 0x5a — the approach chase [M `NDrone2_DSTATE_CombatOutOfRange` 0x164690:38-48, the CombatState router];
   - COMBAT_WAIT 0x5e — the bunching wait (`DroneMove_NoBunching` parks the mover here [M `DroneAnim_SetCombatMoveAnim` 0x13cfb0:35-39]);
   - AIM_BACKOFF 0x6d [M `NDrone2_CanBackoff` 0x1501a0];
   - the strafe family 0x73-0x7a (step/strafe/dodge/roll pairs [M `NDrone2_ChooseCombatMove` 0x150c18]);
   - the scare cover-seek 0x86/0x87 [M 0x168aa8/0x168d18] and UNDER_COVER_RETURN 0x99 (walks back to cover while > 1.5 away [M 0x16a9c0 case 3]);
   - plus the **combat-band anim check**: a guard in a non-mover combat state (0x56, 0x58-0x5f, 0x67-0x71, 0x73-0x7a) playing a `DroneAnim_SetCombatMoveAnim` move anim (0x04/0x08/0x0c/0x0d/0x0f/0x29/0x2a/0x53-0x56/0x5a/0x5b, the RUN/WALK fallback [M 0x13cfb0:75-392]) counts as wanting travel. HIT rows now carry the anim id.
2. **The 07000005 real-input walk** (`crates/nf-view/tests/stuck_guard_combat_05_live.rs`):
   the cold start #26 → the road (the waypointed lane the first run's nav legs measured:
   62,26,96 → 48.5,28.2,96.2 → 37.6,30,93.9 → 32.2,31.8,88.4 → 30.5,32.6,85.1 → 30.1,36.2,66.3
   → the truck park 29.85,37.74,50.30 [D the road climbs 25→37.7]) → west along the wall → the
   pad attempt. **The helipad courtyard itself was NOT reached**: every pad-side leg wedges on
   the ledge above (Bond y 34.5-34.7, the pad courtyard floor ~37.5+, the parked heli statics
   (−17.0, 43.0..44.8, −37.2..−43.6) [D fix-portal5 §6] higher still; the walker's descent/wedge
   at (34.64, 34.72, 49.58) and (25.4, 34.6, 49.8) — floor_below 28.93-33.45 under a y-34 Bond).
   The trigger-gated pad guards (drones #7/#10 at (−31.3, 37.5, −37.05)/(−32.77, 37.5, −22.32)
   [L the load digest]) were engaged through the game's own messages from the gate area: msg 0x0e
   ENABLE (WaitSwitch → the start state [M the msg 0x0e handler]; 6 guards) and msg 0x1e ALARM
   [M `DroneVision_EnemyAlerts` case 0x1e 0x177c18:113-114]. The measurement runs 60 s of ticks
   with the extended moving set; every HIT row dumps the drone's route/cover mechanism
   (croute path/index/remaining/target, cover node/point, goal).
3. **The frozen-step class found and fixed.** Run 1 (the single nav legs, Bond wedged below the
   gate plateau) froze **EIGHT guards in STEP_AIM_LEFT/RIGHT** (0x73/0x74, anims 0x5a/0x5b)
   inching < 0.1 per 3 s for the whole 60 s window (drones #0/#1/#2/#3/#5/#6 + the pair at
   (53-56, 27, 96-97)), plus drone#6 RUN_FOR_COVER frozen 13.53 from its goal. Mechanism: the
   combat-move **CHOICE** probe (`can_move`, the knee/chest rays at +0.5/+1.4) passed where the
   **MOVER** (`move_by`, the feet/knee rays at +0.0/+0.5 plus the 0.4 overshoot) rejected — every
   completed step anim re-picked the same blocked step (0x73/0x74 ↔ 0x68 forever). The game
   cannot diverge this way: its choice probe (`NDrone2_CanStepLeft` 0x14f640 family) and its
   mover share the one `NDrone2_MoveTest` + `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0
   machinery [M]. **Fix**: `Drone::probe_move` (move_by's test as a dry run) now gates the
   choice's pair(), the evasive dodge probe and the strafe UPDATE's blocked exit
   (crates/nf-game/src/drone.rs) [S the ray set itself is unverified; the choice/mover agreement
   is the port's own requirement].
4. **Sister**: `no_guard_loops_a_blocked_combat_step` (same file) asserts the class invariant on
   the real walked path: no guard sits in 0x73-0x78 with < 0.1 movement over 3 s. **Honest
   caveat**: it passes post-fix on every flow run, and it ALSO passes pre-fix on the three repro
   flows tried (alarm-only; the waypointed walk; the ledge + pad-attempt flow) — run 1's exact
   chaos (its RNG state at the alarm tick) is what froze the eight; the invariant assertion is a
   regression guard for the class, not a proven repro.

## What's left

1. **The COMBAT_NO_SIGHT 0x5c sight-loss wall (the next class, measured precisely).** Run 2
   (post-fix, the waypointed walk) parked **SEVEN guards in 0x5c** (the RUN_AIM anim 0x08 / walk
   0x0f playing) at (28.15, 37.00, 48.06-48.10) — `unseen_ticks` 18,209-27,779 (5-7.5 min),
   croute remaining 3.25-3.27 frozen, route target = Bond's feet (25.31, 34.72, 49.70) **1.8 u
   below their ledge**, route_blocked false. This is exactly the sight-loss exit the port has
   disabled: `if false && matches!(from, COMBAT | COMBAT_OUT_OF_RANGE | COMBAT_NO_SIGHT |
   AIM_STAND_FIRE | AIM_CROUCH_FIRE) && !self.seen && self.unseen_ticks > 15 { return
   NO_OPPONENT; }` (drone.rs:9866, off [S] — the 15-tick stand-in for the +0x238 hold stalled
   searchlight/abseil/door-chase/bathhouse guards on 06/09/16). The game's rule: `HasOpponent`
   0x14ad58 — opponent `+0x170` != 0 OR `GameState+52 <= +0x238 + 0x1e` (the 30-frame
   last-seen stamp [M 0x148338's router, 0x14ad58; 0x175ef0:66-67 writes the stamp]) — the port
   needs the real +0x238 stamp before the exit can come back on. (The prior
   combat-sight-loss-exit task owned this; it was superseded before landing.)
2. **The pad courtyard walk**: the ledge descent (y 37 → 34.7 → the pad approach) wedges the
   walker; the pad guards' own spawn plates (#311/#312/#313 out 127/128/129, #314 out 133, #253
   out 139, #315 out 164 [D docs/levels/07000005.md §3]) were never tripped by walking. A
   descent-capable walk (or the game's own ramp traversal per level-05-west-lane-7's findings)
   is the follow-up; the ENABLE stand-in covered the engagement this round.
3. Run 1's drone#6 RUN_FOR_COVER freeze (goal 13.53 away, anim 0x5a, route_blocked false) was
   not re-measured post-fix (its flow is not reproducible standalone — see the sister caveat);
   if it survives the sight-loss fix, it is its own round.

## Verification (what I ran)

- The full suite (`tools/test/suite.py`, 776 test binaries): **running on this tree at report
  time** — build ok in 916 s; its final total line is in `suite.out` at the repo root. Two
  reds observed mid-run, both pre-existing this morning (the 08:31 suite.log on the prior
  branch): `stdio_pipe_closed_live::a_closed_stderr_pipe_does_not_crash_the_game` FAILED
  (panics at stdio_pipe_closed_live.rs:28), and `net_desync_report_live` CRASH (exit -9 =
  SIGKILL, almost certainly OOM while my parallel verification binaries ran beside the suite).
  Neither touches nf-game AI.
- On the fixed tree, directly: `stuck_guards_sweep` (nf-game) ok 16.8 s; `guard_move_checks_live`
  ok 4.1 s; `walkreal_07000005_live` 6 passed / 2 ignored; `guard_stuck_sweep_live` (the extended
  sweep) 0 hits across all 17 on-foot SP levels over 60 s each; `stuck_guard_combat_05_live`
  (measurement + sister) 2 passed in 93 s. Pre-fix sister checks ran against a stashed tree
  (documented above).

## Live checks for the main agent

1. The 0x5c sight-loss hold, fresh boot, 07000005: park Bond out of a guard's sight in combat,
   `gdbwatch` Drone+0x238 (the last-seen stamp) and Drone+0x228 bit 2/4 (LOS/seen): expect
   +0x238 to hold the last seen FRAME and the state to leave 0x5c for 0x60 (NoOpponent) once
   `GameState+52 > +0x238 + 0x1e` — i.e. the game does NOT park a sight-lost guard minutes.
   The port's frozen seven are at (28.15, 37.00, 48.06-48.10) after the walked engagement.
2. A guard stepping against a kerb in combat (0x73/0x74): whether the game's guard inches
   (root motion + bounds push) or turns away — the port now refuses the blocked step at CHOICE
   time; confirm the game's guard does not loop the same blocked step either.

## Usage

Not visible to me: this sandbox exposes no token, credit or context-window figures for the
session (no environment variable or file in the sandbox carries them). The driver message's
token is used only as the git HTTPS password.
