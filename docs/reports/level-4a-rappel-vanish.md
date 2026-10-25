# level-4a-rappel-vanish — the 0700004a shot rope guard falls instead of vanishing

Branch `s/level-4a-rappel-vanish` (from master 24084ef2). Sandbox deadline ~22:14 UTC; this run
started 20:15 UTC.

## What was wrong

Real play on the 0700004a Tower-2 lift ride: shooting a rappelling guard made him VANISH the tick
he was hit instead of dying/falling. Reproduced through the real input path (the ride driven with
`step_bond`, real trigger cadence): abseiler #282 wakes on the shaft plate's ch 0x3d [D], ropes
(Init 0x8e → Slide 0x8f), Bond's rounds kill him mid-slide → state 0x92 (AbseilDeath) with
`present = false` THE SAME TICK while the per-tick cull said `drawn = true`, grace 2.

The port's AbseilDeath UPDATE arm for 0x700004a was doubly wrong:

1. **The delete gate was inverted on the wrong field.** The game's UPDATE keeps the corpse while
   it has been drawn recently — `if (obj+0xFC != 0) return` — and only when the drawn-grace
   counter has run out calls `NDrone2_Enable(0, drone)` + `obj+0xFE |= 1` (the delete-me flag,
   the movement pass then removes the object) [M NDrone2_DSTATE_AbseilDeath 0x169b28 case 3,
   lines 41-47]. obj+0xFC is the drawn counter: 2 per drawn tick [M View_AddObjects 0x1e62d8:71],
   −1 per control pass otherwise [M 0x1336f8:242-246] — a corpse is removed on the 3rd undrawn
   control pass (docs/systems/corpse-persistence.md). The port read the per-tick cull BOOLEAN
   (`drawn`) and deleted when it was TRUE — a guard you are looking at (i.e. one you just shot)
   vanished immediately.
2. **The corpse never fell.** AbseilDeath's ENTER on 0x700004a sets `+0x4F8 = (+0x4F8 & ~0x20) | 4`
   [M 0x169b28:18-21]. With bit 0x20 clear `NDrone2_DoCollision` 0x149ec8:12-14 returns 0 — NO
   collision (no down-ray, no settle) — while `NDrone2_DoGravity` 0x141e58:12-15 returns 1 on bit
   4 BEFORE the 0x20/0x200 folds, and `Drone_CollisionHandler` 0x140968:152-192's DoGravity arm
   integrates the corpse collision-free: velocity += WldGravity·REC_FRAME_RATE (the feet flag
   anim+0x60 & 8 stays clear — no probe runs in that branch), pos += velocity·REC_FRAME_RATE, no
   damping (the ×0.98 tail is the rope arm's, [M 0x140968:139-141 vs 152-192]) and no settle
   [M 0x140968:292-295 — the tail's feet-flag test never fires]. The port's `bounded = false`
   (its stand-in for bit 0x20) skipped the WHOLE handler — velocity never integrated, the corpse
   hung in mid-air until the (inverted) delete fired.

## The fix (`crates/nf-game/src/drone.rs`)

- New field `keeps_gravity` = `Drone+0x4F8 & 0x4` (the 0700004a dead-body-keeps-gravity bit
  [M docs/fields/drone.md Group 1: Death_Anim 0x1637f8:26 and AbseilDeath 0x169b28:20 set it,
  DoGravity 0x141e58:13 returns 1 on it]). Set by the AbseilDeath ENTER's 4a arm next to
  `bounded = false` (bit 0x20).
- `collision_handler`: with `bounded == false` (bit 0x20 clear → DoCollision 0) a
  `keeps_gravity` corpse integrates the game's collision-free gravity arm exactly (velocity +=
  WldGravity·(1/60), pos += velocity·(1/60), no damping, no settle, no floor probe). Other
  `!bounded` cases keep the old early-return.
- AbseilDeath UPDATE's 4a arm now gates on `drawn_grace == 0` — the obj+0xFC counter, read before
  this tick's refresh exactly like the game's control-pass ordering — and soft-deletes
  (`delete_me`/`present=false`/`enabled=false`, the port's Enable(0)+obj+0xFE|=1 fold, the same
  triple the DEAD state's grace arm uses [M 0x163fe8 case 3]).

The non-4a arm (07000011 et al.) is untouched — `abseil_death_live` (nf-game) and
`abseil_elevator_live` (nf-view) pass unchanged.

## Verification

- **Real-input repro test** `crates/nf-view/tests/rappel_vanish_0700004a_live.rs`:
  `a_shot_rope_guard_falls_instead_of_vanishing` drives the lift ride from the spawn with real
  input (the fix_finale_live platform re-seat), wakes #282 on ch 0x3d, kills him on the rope with
  the semi-auto trigger cadence (3 pressed / 7 released, aim re-set per volley), then pins:
  - the kill routes to AbseilDeath 0x92 with the rope released [M 0x1461c8:44-52, 0x169b28:28];
  - the corpse is present and drawn the kill tick — the vanish is gone [M 0x169b28:41-44];
  - the fall integrates the game's exact gravity for the first 12 ticks (velocity −9.8/60 per
    tick, no damping) [M 0x140968:173-191];
  - the corpse never lands (on_surface stays false — DoCollision 0) [M 0x149ec8:12-14] and falls
    > 25 units down the shaft while drawn;
  - the delete comes only after ≥ 2 consecutive undrawn control passes and > 60 ticks after the
    kill [M 0x169b28:45-46 + 0x1336f8:242-246].
  A per-tick log (state/danim/pos/velocity/drawn/grace/present) prints under `--nocapture`.
- `the_disc_abseil_census_at_difficulty_1_and_2` pins the disc census: at difficulty 1 exactly
  two 0x06000892 abseilers spawn — #282 (−1.605, −7.518, −4.490) ch 0x3d and #283 (−1.226,
  −95.565, −4.645) ch 0x4f; #288 (−1.537, −50.149, −4.861) ch 0x19 is the difficulty-2 spawn
  [D placements; M Drone_Create 0x136be0:49]; no 0x0600089c placement [D].
- Regression: `abseil_death_live` 5/5 (nf-game), `abseil_elevator_live` 3/3 (nf-view) — the
  07000011 rope-death path (corpse lands, danim 10, DEAD 0x47) unchanged.

## Full suite

`NF_ISO=… python3 tools/test/suite.py` on this tree (master 24084ef2 + this fix):

- See "Suite result" below (filled from suite.out / suite.log at the end of the run).

## Headless renders for the main agent

All in 0700004a, the repro geometry (Bond rides at [3.0, platform+1.2, −3.4]; the guard ropes at
x −1.6, z −4.5; y is Bond's pos):

- `--bond 3.0,-27.3,-3.4,-1.805,0.848` — the kill moment: the roping guard visible above/beside
  the car, ON the rope (state 0x8f), gun out.
- `--bond 3.0,-27.3,-3.4,-1.842,0.708` — ~1 s after the kill: the corpse MID-FALL below the rope
  point (the death anim playing), NOT vanished. Before the fix this frame showed an empty shaft.
- `--bond 3.0,-27.3,-3.4,-1.809,-0.807` — later (the corpse > 25 units down or after the grace
  expired): the shaft empty again — the game's soft delete, not the vanish bug.

## Live checks for the main agent

None required — the behaviour is pinned by decompile [M]. Optional (strengthens [L]): in PCSX2 on
0700004a, shoot a roping guard while watching him, then look away: `gdbwatch` the drone's
`obj+0xFC` (expect 2 while drawn, counting down once out of view, the object removed at 0 —
~3 control passes) and `obj+0x4F8` at the death tick (expect bit 0x20 cleared, bit 4 set).

## What's left

- Nothing known. If the full suite surfaces a red outside the 3 allowed slim-ISO `disc_golden`
  fails, it gets a follow-up entry here.

## Suite result

(pending)

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).
