# level-11-start-corridor-guard-2 — report

Branch `s/level-11-start-corridor-guard-2` (from `origin/s/level-11-start-corridor-guard`, then a
merge of `origin/master` — merge commit 2249e9f5). WORKLOG: `docs/WORKLOG.md`, entry
`level-11-start-corridor-guard` (sessions 1-2, the -1 task) + session 3 (this task).

## The task

The -1 task fixed the run-to-point owner read (Drone+0x144 = placement param 4 [M DefaultInit
0x14b300:472]) but the pin was not met: at d3 the port's guards #127/#126 still walk into the start
corridor and kill an idle Bond by ~tick 700, while the PS2 capture shows the corridor empty over
f200-f3172 [L]. This task: find the remaining divergence among (the point's own state/latch, the
0x4000000 latch, LOS from the points, the capture's difficulty / GameState+0x40), fix it with the
address cited, replace the branch's EXPERIMENT comments, and pin it.

## What the data says — every lead resolved, no port fix needed

I read the game's path again end to end; the port is data-faithful at every step I can see, and two
of the leads die on the disc bytes:

1. **Prop 0x56 (the RunToAlarm redirect) is clear.** `EnemyRunToPoint`'s ENTER first asks
   `behaviour_util_getProperty(0x56, props)` and, when it is non-zero, skips the run-to-point
   machinery entirely and goes to RunToAlarm 0x89 [M 0x168910:17-27]. Property 0x56 decodes through
   `bitDescs[172/173]` = word 2, shift 26, mask id 26 → `bitMasks[26]` = 0x4000000 [M 0x1c7d98; the
   tables at ACTION.ELF 0x2c7250/0x2c7308 [D]]. Both guards' word 2 (param 20) = **0x40280000**
   (bits 30, 23, 21, 19 — bit 26 clear) and the alert block's word 2 (param 25) = 0x40291660 (also
   clear) [D 07000011.bin map 43 params 18-25]. So the game takes the same mode 7 → start 0x85 →
   run-to-point path the port takes, and my first hunch (a RunToAlarm/Idle fork) was wrong — a
   probe run of the port caught it before I "fixed" anything (props words and prop read both
   confirmed; mode 0x7, state walk 0x85 → 0x63).
2. **AIPoint+4 (the eligibility byte) never blocks.** `ClosestAIPoint` requires `*(char*)(pt+4) !=
  0` [M 0x151b18:128], and `Drone_AIPoint` writes **1** there for every point at creation [M
   0x137550]. The port's `aipoint::closest` doesn't model the byte because it is uniformly 1 at
   boot; nothing to fix.
3. **The 0x4000000 latch is clear at boot.** Only the sighting paths set it (HandleImpact's
   FirstAttack latch [M 0x1469b8, the port's `sighted`]), so the Tower clause applies [M
   0x168910:24-31] and at alertness 1.0 the resume stays ATTACK — which the port does (probe:
   return_state 0x56).
4. **The difficulty gate is `<=` and GameState+0x40 is the difficulty.** `Drone_Create` creates a
   drone iff `placement p2 <= GameState+0x40` [M 0x136be0:49]; +0x40 reads as a 0..5-band
   difficulty in `NDrone2_HitDamage` [M 0x145558:135-144] and `DefaultInit` [M 0x14b300:505/:514/
   :526]. At difficulty 0 **neither guard is created** (p2 = 1 and 2 fail `<= 0`) — that is the
   game's own empty corridor. The port's gate matches (the d0 test below passes).

So the remaining divergence is not in the AI data path at all. The two facts that can still empty
the corridor in the game are both [L]-only (see Live checks): **what difficulty the f200-f3172
capture ran at** (at d0 the corridor is empty with no port change at all), and **whether the game's
per-cel enable bytes hold the guards' control frozen** at their spawns — every object inherits
+0x100/+0x101/+0x102 from its cel's +0x9a/+0x9c/+0x9e (indexed by `switch_channels[obj+0xa2]`) at
every cel link [M 0x1336f8:292-296, 0x1346b8], and no reader that gates the drone AI on them is
located yet (the -1 session's open hypothesis; the disc's cel bytes would be readable once the
reader names them).

## What changed

- `crates/nf-view/tests/start_corridor_guard_07000011_live.rs` — now 5 tests, all green:
  - the -1 task's two (the sister: both guards present at d3 with their skins + param-4 owners;
    the wake chain; the owner pairing; the old props-bit read as a negative);
  - **new** `prop_0x56_is_clear_and_no_alarm_point_exists`: the disc words, the decoded bitDescs
    row, and `ClosestAIPoint(…, 8, …)` = None — no kind-8 AIPoint exists on the level [D the 12
    map-43 0xE9 placements, p0 ∈ {0,1,2}], so RunToAlarm's Idle fork is not on this level either;
  - **new** `at_difficulty_3_the_guards_walk_their_disc_owner_points`: by t400 both guards sit
    within 2.5 of their owner points (1.40, 16.70)/(−1.51, 18.21) [D; M 0x151b18] with the resume
    state ATTACK (the Tower clause [M 0x168910:23-31]) — the port's d3 walk is the game's own data
    path, pinned;
  - **new** `the_corridor_stays_empty_at_difficulty_0`: at d0 no guard satisfies the create gate
    [M 0x136be0:49] and no drone comes within 8 units of the idle start over 3200 ticks — the
    empty-corridor capture, pinned at the difficulty the data supports.
- `crates/nf-game/src/drone.rs` — the two `EXPERIMENT (level-11-start-corridor-guard)` comments in
  `find_alarm_point`/`find_run_to_point` are now plain cited comments (the owner = placement param
  4 [M 0x14b300:472]). No behaviour change (release build + the tests above confirm).
- `crates/nf-formats/examples/dump_07000011_drones_aipoints.rs` — the [D] probe that produced the
  param dump (drone placements with behaviour words 18-25 and every 0xE9 AIPoint), kept as a census
  example like its siblings.

## What's left / Live checks for the main agent

The literal pin ("idle Bond at the start is not approached in 3200 ticks" with "both guards
present") needs one game fact I cannot read from the disc: whether the game's guards run their AI
from tick 1 at d2/d3. Two live checks settle it:

1. **What difficulty did the f200-f3172 capture run at?** If it was d0 (Operative), the corridor is
   empty with no port change — [M 0x136be0:49] creates no guard (p2 = 1, 2 > 0) — and the pin is
   already satisfied by `the_corridor_stays_empty_at_difficulty_0` at the matching difficulty. A
   fresh-boot capture at d2/d3 with the corridor still empty would instead confirm the cel gate.
2. **Do the guards' AI run at boot?** Fresh boot 07000011 at d3, Bond idle at the start; gdbwatch
   on drone #126/#127's `Drone+0x30` (pos.x) and the SM state byte for f0-f400. Expected if the
   per-cel enable gate holds them: pos constant at 10.43/−11.39 and the state never leaves 0;
   expected if they run: the state walks 0x85 → 0x63 by ~t60 (the port's timing). If they run,
   also fieldprobe `obj+0x100/+0x101/+0x102` of #126/#127 at boot (the inherited enable bytes
   [M 0x1336f8:292-296]) — if any is 0 while the cel's +0x9a/+0x9c/+0x9e (channel-indexed) differ
   from Bond's cel's, that byte pair is the AI gate and I can port it with the reader named.

## Runs

- `cargo test --offline --release -p nf-view --test start_corridor_guard_07000011_live --
  --test-threads=1`: **5 passed; 0 failed**.
- Every `*07000011*` binary (16, nf-view, `--test-threads=1`): **49 passed; 0 failed; 8 ignored**
  (walkreal 14, walkreal_route 3+1 ign, interactables 3, walk 2+1 ign, start_corridor_guard 5,
  stairwell_guards 3, stairwell_walls 1, stairwell_ghosts 1, decal_door_follow 4, models_sweep 2,
  rope_guard_hitbox 6, stairwell_bullets 2, stairwell_decal_interp 2, stairwell_decal_probe 0+5 ign,
  stairwell_impacts_render 1+1 ign, stairwell_walls4_scope 0+1 ign).
- `cargo test --offline --release -p nf-game -- --test-threads=1`: **987 passed; 0 failed; 9
  ignored**.
- The fast runner (`tools/test/suite.py`, build cached from this session): started on the final
  tree; see `suite.out` / the ntfy line for the total if it finished before the report cut — the
  per-binary totals above are the verified numbers this session pushed behind. (Background
  processes were reaped between tool calls twice earlier in this sandbox; the runner was started
  detached for the final tree as required.)

## Usage

not visible to me
