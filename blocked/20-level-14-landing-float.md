# 14: Bond floats over the landing

`walkreal_07000014_live walk_the_infiltration_to_the_delta_exit_real_damage_00agent` wedges at tick 26340: Bond floats 0.41 over the floor at (33.84, 0.82, 30.95) on the 1.72 landing resupply leg. The earlier task on this closed with no report and no code.

Treat the float as a possible game-side bug first. Put Bond at that spot on master, settle, and print grounded, the floor ray and the capsule contacts. If he hovers there without the walk, fix the collision and cite the decompile. If only the walk causes it, say which leg and fix the route. Remove the ignore when green and quote the whole binary.

---
claimed-by: sandbox4 at 2026-11-06T08:59Z

---
## Report (blocked, 2026-11-06T09:57Z)

# level-14-landing-float — the 1.72-landing hover: game-side, fixed at the pocket; the d3 walk's remaining blocker is the infiltration grind's budget

Branch `s/level-14-landing-float` (from origin/master 62c8281d6). The task: the recorded
`walkreal_07000014_live walk_the_infiltration_to_the_delta_exit_real_damage_00agent` wedge at
tick 26340 — Bond floating 0.41 over the floor at (33.84, 0.82, 30.95) on the 1.72 landing
resupply leg.

## What's done

**1. The float reproduced standalone — it is a game-side (port-side collision) bug.**
New probe `crates/nf-view/tests/probe14_pocket_settle.rs` (assertion-free, `#[ignore]`d): load
07000014 at d3, settle at spawn, teleport Bond to the recorded spot, settle 240 quiet ticks.
Measured on this tree: Bond parks at (33.9712, 0.6500, 30.9501), `grounded=false`, `hist 0x5`
(grounded flickering every other tick — the dither), for the whole 240 ticks, no input. The
holder: the port's **[S] spine-fallback push** `(0, +0.0027, 0)` every tick off the flat y≈0
floor (`n = (0,1,0)`, cent (34.320, 0.000, 32.067), `d = 0.5473` — the r = 0.55 equilibrium).
The game's own push tail is DEAD in exactly this class [M Intersect_CylGeom 0x109da0:431-447]:
the whole push sits inside `0 < dot(q − HITTEST+0x50, n)` AND `|q − ref| < radius`, ref = the
lifetime DeltaP accumulator [M Collide_Intersect 0x1ebaa8:353-373; L ee_start_* boots (0,0,0);
L ee_snapshot drifts ~15 m] — for a face under the capsule q is below ref and the dot gate
fails; there is no fallback in the decompile. The port's fallback (dir = v̂ ≈ +Y, mag =
r − dist) is the invention that holds the capsule at the r-equilibrium forever. The walk's
floor ray at the pocket reads the y≈0 surface (the scoped hit sits 0.65 below the root — t
0.325 of the walk's 1.25 ray), so the band needs dy 0.65 ≤ cc + 0.1 and fails by an ulp while
floating; walkable stays false through the fall (the descent's pusher is the pocket's west
skirt, n = (0.906, 0.423, 0), cent (33.427, 0.400, 31.133) — n[1] ≤ 0.7, never walkable).
Diagnosis instrument: `NF_WALK_DBG=1` (scratch print of the walk pass's per-tick
pos/vel/floor/dy/cc/walkable/band/push/grounded) + the existing `NF_CAPS_LOG=1`.

**2. The fix: the failed-ref up-facing hold skip, scoped to the quiet-dither regime, walk
path only** (`crates/nf-game/src/world.rs` `capsule_tri_contact`, new `walk_flat_hold` param;
`capsule_push_scoped_hold` / `capsule_push_hold` carry it; `player.rs`'s walk pass calls the
`_hold` variants — the climb query (player.rs 1087), the pick-contact queries and the
rigid-body sweep keep master behaviour). Rule: when the radial (game-shaped) arm's gates
fail, the face is up-facing (`n[1] > 0.7`), q is below the spine bottom, and the fallback
mag is **< 0.02** — skip the displacement; keep the contact and the walkable/pick signals.
The mag gate is the measured scope: the first (mag-less) version broke the stairwell
pit-band jump at tick 1380 ("the pit band is jumpable east back onto the corridor" — the
big-mag fallback lift over the band is load-bearing for the walks' jump arcs), while the
hover equilibrium's mag is 0.0027. After: the pocket grounds — `grounded=true`, `hist 0xf`,
vel 0, rest (33.9712, 0.6335, 30.9501) — and the d3 walk clears the stairwell and the pit
band it used to fail at 1380.

**3. The two prior lever branches are carried on this branch** (cherry-picked, conflict
resolved to keep both sets): `s/level-14-ignored-walks` ae4f15cca (the d3 trench
return-fire stand, the bounded drop loot pass, the wheel draw — all d3-gated thread-locals
in this level's test file) and `s/level-14-landing-hover` d96f7863b
(`DamageLog::descent_stationary_fights` — shared helper, opt-in, set for every
`difficulty >= 2` body call and cleared after the descent block; `escalating_cooldowns`;
the d2 `with_real_damage` head-hunt).

**4. Measured state of the d3 walk** (`--exact --ignored`, release, this tree): the recorded
26340 wedge is NOT the failure any more — the walk now dies at **tick 6437** by real-damage
BUDGET in the infiltration grind, the same shape the landing-hover window measured at 6786:
31 hits for 100.0, 16 return-fire fights; the far guards #9/#10/#3/#11 (hp 10 each, d
1.1–4.0) take 65.0 + 15.0 + 10.0 + 10.0 through stands that land ZERO rounds (`killed
false`). The `eye_ray_stands` lever (the ignored-walks branch's named next lever, the
07000011 law) did not change it — the rounds die on the corridor corners (geometry), not on
the aim pick. **The ignore therefore stands**; the un-ignore is not landed.

## How it was verified

- `probe14_pocket_settle` before/after (above), run with `--ignored --nocapture
  --test-threads=1`, `NF_CAPS_LOG=1`.
- The d3 walk test binary run for the one target test, twice: with the unscoped fix (1380
  pit-band fail — the regression), then with the mag gate (6437 grind death — the
  stairwell regression is gone, the recorded 26340 wedge is gone).
- Per-binary counts this window: walkreal_07000014_live — the target test FAILED (6437
  death, above); the other 9 tests in the binary were NOT re-run this window (time box).
  `walk_flat_hold` changes master behaviour on every level's walk — **the full walkreal
  battery has NOT been run and is required before merge** (lessons 7/10). Highest-risk:
  any walk whose jumps ride the big-mag band lift (kept by the gate) or whose settle
  escapes use small-mag up-facing holds (skipped now).

## What's left

1. The infiltration grind's fight answer: the geometry-blocked far guards land 31 hits for
   100.0 while the stands land zero rounds. Levers, in order: the 07000013 repositioning law
   (walk the answer from a spot whose round path is clear — the back_off band is already
   4.0), or the answer-only-after-first-damage gate (stop proactive fights against guards
   that never dealt damage — 16 fights before the death), then re-measure.
2. The whole `walkreal_07000014_live` binary green, the ignore removed, the whole binary
   quoted (the task's ask).
3. The full walkreal battery + the door/guard tests (the shared-helper + world.rs rules).

## Live checks for the main agent

1. The pocket rest height: on PCSX2, fresh boot, walk/teleport the game's Bond into the
   pocket at (33.97, ~, 30.95) and read obj+0x30 (root y). The port's fixed rest is 0.6335
   (band rest; the pre-fix hover was 0.65 with the fallback dither). Expected [S]: the
   game's Bond STANDS on the y≈0 floor (root ≈ floor + the crouch band rest ≈ 0.65),
   grounded, no hover. Access: BLData obj+0x30; watch `grounded` (obj flag 8) settle to 1.
2. The hold-skip's blind side: the game's Bond jumping the 07000014 stairwell pit band
   east (from the −0.91 landing edge x≈46.1 to the corridor x>47.6) — confirm the game's
   arc needs NO mid-air push (the port's big-mag lift compensates a lower arc; if the
   game's arc is genuinely higher, the port's jump physics are the [S] gap to close
   instead).

## Usage

- Session token/context/cost figures: not visible to me (no token or cost display is
  exposed in this environment).
