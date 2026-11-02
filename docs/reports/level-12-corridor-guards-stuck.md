# level-12-corridor-guards-stuck — guards stuck at the security-center corridor

## What the task was

On 07000012, from Bond at (22.49, 1.09, 19.16) facing yaw −1.56 at the start, the two guards at
the far end of the corridor (by the exit sign, near the dropped case) stay bunched in place and
walk into nothing instead of coming down the corridor. Reproduce headless, name the blocker,
fix the port where it differs from the game, pin with a live test.

## What's done

**Setup.** Fresh sandbox: the repo cloned to `/home/z/NightfirePort`, branch
`s/level-12-corridor-guards-stuck` created from `origin/master` 88d87c9c and pushed (checkpoint
60708ef2); `work/` (work-slim.7z) and `Nightfire-onfoot.iso` (1,061,754,880 bytes) in place;
rust 1.99.1 + py7zr installed (no root, so 7z came from py7zr and rustup from the static
binary). The queue worktree was already claimed (`10-level-12-corridor-guards-stuck`, sandbox5).

**The headless repro is committed** (`crates/nf-view/tests/corridor_guards_07000012_live.rs`,
log-only, diagnostic): loads 07000012 through the real path, dumps the corridor drones, runs LOS
probes at five heights along the corridor axis, dumps the AI-volume visibility factors, the nav
path membership and the BLINK walls, then (phase 1) walks Bond down the corridor with the real
input path and (phase 2) wakes the guards through the game's own alarm path
[msg::ALARM — `DroneVision_EnemyAlerts` case 0x1e, +0x4F8 |= 0x4000000|0x40000000, SetState(0x56)
[M 0x177c18:113-114]] and logs 600 ticks of state/pos/anim/goal.

**Named the two guards** [L the port's load dump, matching enemies_07000012_live's
dump_every_drone_at_load]: drone 6, skin 0x05000008, placement 12, (13.09, 1.00, 18.55), ALERT
0x05, no start script, watching; drone 7, skin 0x05000006, placement 13, (12.98, 1.00, 16.43),
start script 0x060008ae (the known PlayScript intro [M 0x15ba68 — ported, drone.rs:7165-7215]),
settle to IDLE 0x04. These are the only enabled watching guards on Bond's corridor.

**What the repro measured (port side, release, fixed 60 Hz):**

1. **The corridor is open between them and Bond.** World LOS from both guards' heads to Bond's
   spawn head is clear at y 1.4, 1.62, 1.9, 2.2, 2.6 (no hit anywhere on the x 13→23 axis at
   those heights). No kind-0 AI volume covers the spawn, the guards or the corridor
   (`visibility_factor` = 1.0). Both guards and Bond's start sit on the same nav path (path 8).
   The BLINK walls around the corridor are the z≈15.1 south run and the z≈19.85 north run with
   an inward jog to z 18.73 at x 17.2–18.5 — none crosses the guards' path east.
2. **The guards never see Bond approach, and the reason is their facing.** At load:
   drone 6 yaw −1.89, drone 7 yaw −1.38; Bond sits at yaw_to_opp +1.50 / +1.29 from them —
   the angle between facing and target is ≈2.89 rad against a `half_fov` of 1.571 (+0.393
   margin) [M the DroneFunc_OpponentVisibility cone gate, drone.rs:11140-11151], so the
   visibility score is 0 every tick. Their sight_range is 12, dist 9.4/9.9 — in range, LOS
   clear, but outside the cone. The game's ALERT does not rescue this: its UPDATE (case 3) is
   `DroneVision_EnemyLookForOpponent` + re-anim 0x21, and the look-around arm needs prop 0x46
   (case 4) [M 0x15c248 cases 3/4] — neither guard has prop 0x46 [L the SIGHT dump]. So both
   stand at their post yaw for the whole 360-tick walk, and Bond's approach never trips them.
3. **The yaw is the disc's own float.** The port's spawn yaw is the placement rotation's second
   angle, read raw from the placement row (+0x1c f32, radians) [D placement.rs:8/:54] with no
   transform — so −1.89/−1.38 are the disc's values, and the model matrices built from the same
   angles render correctly everywhere else. The open question is the GAME's drone-yaw
   convention: `Drone_Create` receives the placement's rotation VECTOR [M
   0x1d04f0 case 0xf → 0x136be0, param_2 = +0x18..0x23] but the decompile drops which component
   becomes the drone's facing (the C shows a 0x84-byte row copy only), so whether the game's
   drone yaw is +rotation[1] (port's reading — guards face west, away from Bond) or the
   negation / another component (guards face east, down the corridor toward Bond's spawn) is
   NOT settled by the decompile text.
4. **When alarmed, drone 6 advances ~2.1 u down the corridor and parks in AIM_STAND_FIRE:**
   0x74 STEP_AIM_RIGHT → 0x5c COMBAT_NO_SIGHT → 0x68 AIM_STAND_FIRE at (15.17, 1.00, 18.82),
   stationary from t=179 to t=299+ with anim 0x0a (a fire-loop anim, not a combat move anim —
   the stuck sweep's `wants_travel` correctly does not flag it). Drone 7 flips to
   0x70 AIM_CROUCH_FIRE and barely moves (0x67 AIM_CROUCH later). The first-sighting coin flip
   at ≥ 6 u [M 0x173280 ENTER — crouch-capable → 0x70, else 0x68] explains both states; whether
   the game then ADVANCES (COMBAT_OUT_OF_RANGE 0x5a approach) or keeps standing fire at ~7.3 u
   depends on the weapon's fire range, not yet pinned.
5. **Bond's own straight walk down the corridor stops at x 19.01** (forward held, x freezes
   19.01 from t=59 on, z creeps 19.20→19.38): his r 0.55 capsule rests on the north wall's
   inward jog (#930/#931, x 17.23–18.54 at z 18.73–18.75) — the corridor's north wall genuinely
   jogs south there [D the BLINK wall list], so the spawn-facing straight leg is not the route
   (the walker routes around; this is geometry, not a port gap, as far as the walls show).

## The named blocker (current best reading)

The two corridor guards stand at their placement yaws, which point away from Bond's spawn; the
port's cone gate then never scores the approaching player, so they never leave their posts and
never come down the corridor. Whether that is a PORT BUG or the GAME'S OWN data depends
entirely on the game's drone-yaw convention (item 3): if the game's facing is the negated
placement yaw, the port's `p.rotation[1]` at drone.rs:11754 is the bug (one sign) and these
guards — and every other guard — face backwards, which the combat tests elsewhere would also
have to contradict; if the game's facing is +rotation[1], the port matches the game and the
user-visible "stuck" is the game's own cone behaviour (the guards wake on sound, on being shot,
or when the player crosses behind them), and the fix belongs to the sound/wake side instead.

## What's left

1. Settle the drone-yaw convention: disassemble `Drone_Create` 0x136be0 (work/elf/) to see which
   placement-rotation component lands in the drone's facing, and/or [L] read a guard's facing
   out of `work/ee_start_*.bin` against its placement row.
2. Apply the fix (one sign at drone.rs:11754 if the game negates; else the wake-side work), then
   ADD the pin the task asks for — both corridor guards leave their start spots and come down the
   corridor when the player approaches, asserted through the real input path, with a sister that
   fails on the wrong result. No pin was added on the unresolved side: the only assertion that
   could be written today would guess the game's facing, which the only-proven-behaviour rule
   forbids; the committed test stays log-only (diagnostic).
3. The full suite run (background processes do not survive between tool calls in this sandbox —
   the suite must go in foreground chunks; the required test binaries below were run instead).

## What I ran (this branch, release, NF_ISO the slim on-foot image)

- `walkreal_07000012_live`: **1 passed, 0 failed** (7.07 s)
- `interactables_07000012_live`: **12 passed, 0 failed** (24.58 s)
- `guard_stuck_sweep_live` (SWEEP_LEVELS=07000012): **1 passed, 0 failed** — 0 HITs over 60 s
  (the sweep's mover set does not include the 0x68 + non-move-anim park this task measured)
- `stuck_guard_combat_05_live`: **2 passed, 0 failed** (29.84 s)
- nf-game and the full suite: **not run** — time. The tree carries one new test file and no
  production-code change, so the risk is confined to the new binary, which was run.

## Live checks for the main agent

1. **The two corridor guards' facing at a fresh boot of 07000012** (no save-state): with
   gdbwatch on the drones' facing fields (the port's `Drone.yaw` source offset — settle the
   offset from the 0x136be0 disassembly first) and on the placements at (13.09, 18.55) /
   (12.98, 16.43): read each guard's facing yaw. Expected if the port is wrong: the game's
   facing ≈ +1.89 / +1.38 (toward Bond's spawn); expected if the port is right: ≈ −1.89 / −1.38.
2. **Do the guards come down the corridor in the game?** Fresh boot, Bond walks east→west down
   the corridor from the spawn with no shots: record both guards' states and positions for
   ~30 s. Expected if they wake: a sighting (their cone catches the player when he rounds the
   x 17.2–18.5 jog or crosses behind), then COMBAT_NEW_SIGHTING → 0x68/0x70 or an approach.
3. **The alarmed advance**: fire one shot at the far wall from the spawn, record whether each
   guard ADVANCES past x 16 toward the player or stands in 0x68/0x70 at ~7 u — pins the
   fire-range branch [M the COMBAT_OUT_OF_RANGE gate] this task could not.

## Usage

not visible to me
