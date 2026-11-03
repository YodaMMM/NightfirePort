# level-0d-real-health-crossfire-3 — report

Branch `s/level-0d-real-health-crossfire-3` (the start commit cut from origin/master cbed0983; origin/master merged in at aa8f67d2 — batch-1103i).

## Status: the un-ignore is NOT achieved, but the two -2 drains are DEAD and the route now dies
## 7250 ticks LATER in a phase the -2 route never reached. The driver's levers landed:
## #10 dies at the strip mouth (+201, ZERO incoming — pool 100→100), the walker-catching roof
## sweep kills ALL THREE west-roof guards (+222/+754/+193 in the final run) and the ride goes
## SILENT (zero roof hits), and the route reaches the annex descent (tick 16287, deepest any
## 0d real-health run has gone) before dying to ONE newly-exposed shooter: #32, the ground
## spawner. The non-damage twin is GREEN on the new route (full mission, 23300 ticks, 0 stuck,
## 0 falls). The port was NOT changed — no nf-game/nf-view/src diff this session; every fix is
## a route (tactics) fix, exactly as the -2 audit predicted.

## What's done

1. **The stand-scan probe (`probe_0d_ray_stands.rs`, assertion-free, `--ignored`)**: 22 candidate
   stands × every live guard in the region, each row printing the eye→chest ray through
   `segment_hit`, `segment_hit_static` AND the bullet-law `segment_bullet_scoped`, plus the
   level's own pickup placements [D] and a shooter census. It rewrote the route:
   - The bay guard **#10** (skin 0x5000093, placement (214.21, 12.09, −289.44)) has a **CLEAR
     bullet ray from the strip-mouth stand (196, −228.5) at d 63.6** while every other shooter's
     ray to that stand stays blocked (g16 0.881, g18 0.521, the roofs 0.83-0.86, the canopy pair
     0.56) — the driver's "take #10 before the climb shots" lever, measured.
   - The deck guard **#16** (0x5000087, (198.15, 14.04, −264.66)) has a CLEAR ray from the
     warehouse-floor stands (d 56.4) and the dock edge (d 61.3).
   - The west-roof **placements** (145.6-145.8) have CLEAR rays from the ladder top (#27 d 70.5,
     #29 d 68.0) — the -2 slab-edge law (0.972-0.974) was measured against the guards'
     **repositioned combat spots** (x ~151.4), not the placements.
   - **The pickups [D]**: kind-3 armour at the start #525 (already taken), on the west roof
     (147.00, 21.76, −252.77), and in the annex (192.82, 10.79, −272.03). No detour was added:
     the pain types 1..7 carry armourFraction 0, so the 50 armour cannot absorb this level's
     bullet damage [M Player_HandlePain 0x1902c8:84-104] — the annex armour sits 6-10 u off-path
     and buys nothing.
2. **THE #10 KILL AT THE STRIP MOUTH — landed, free, repeatable** (3 consecutive runs): the
   route stops at the blind-transit leg-2 stand it already walks to, `kill_guard_at_present`
   fires the aimed ray at the woken guard, **down at +201 ticks with pool 100→100**. The
   enabled-gated box found nothing on the first try (the first run's box miss cost the whole
   leg — the present-not-enabled finder `kill_guard_at_present` was added for the
   placement-held bay guards; bullets hit present drones regardless of the AI gate, the AI gate
   only decides whether the guard shoots back — measured: zero incoming across every #10 fight).
3. **THE WALKER-CATCHING ROOF SWEEP — all three west-roof guards die.** The measured chain:
   the wake (the platform-drop trigger, [L probe part E]) sends each guard off its placement
   toward its stable combat spot at x ~151.4 — the slab-shadow band where the -2 measured the
   0.972-0.974 ray deaths; settled guards there are UNSHOOTABLE from the ladder top (11
   straight failed 300-tick passes across two runs). But every pass-1 across the session landed
   (+199/+200/+201): it caught a guard MID-REPOSITION whose ray still clears. The final sweep
   runs the moment Bond tops out (before the #16 fight's aim overhead), 3 passes × cap 900, box
   (151.2, −246.8) ±(6, 11) over the whole wake/reposition band: **down at +222, +754, +193 —
   the ride took ZERO roof hits** (the -2 route ate 54 there from the single survivor; two
   survivors ate 94 in the mid-session runs).
4. **The retired experiments, with measurements** (each cost a run):
   - The (178, −212) warehouse #16 stand: OFF the walkable floor — the walker slid off the
     dock's south edge into the trench (fall 9.6 + 14 in hits, STUCK). Retired.
   - The dock-edge #16 fight (178, −206.5): 600 ticks, health 10 untouched — the live guard
     sits behind deck structure the placement-ray probe missed. The kill returned to the
     ladder top (+209, every run).
   - The trench #18 detour (208, −256): 400 ticks, nothing — the ray to the PATROLLING #18
     dies where the ray to its placement cleared, and the detour's ~1200 ticks let the roof
     walkers settle (the next sweep caught 1 instead of the eventual 3). Removed.
   - The in-bay #18 fight (from the bay-enter stand (194, −282)): 400 ticks, nothing — the
     patrol spot vs the box, still open (below).
5. **The route code is additive and walk-safe**: kill legs + comments in
   `walkreal_0700000d_live.rs` only; `crates/nf-view/tests/common/` untouched this session (the
   OPT-IN `DamageLog::aimed_fights` flag the driver asked for already exists — the 0d walk now
   sets it, the 0700000c precedent). **The non-damage twin is GREEN on the new route**: the
   full mission walked + completed in 23300 ticks, 0 stuck, 0 falls.

## How it was verified

- `walkreal_0700000d_live` `walk_arrival_to_the_c5_door` (non-damage): **1 passed** (18.06 s,
  the full mission, 0 stuck, 0 falls) on the final tree.
- `walkreal_0700000d_live` `walk_arrival_to_the_c5_door_real_damage`: **un-ignored, run 6
  times**; the death advanced 5964 → 4072 (the first #10-box miss) → 6778/7270 → **13139 →
  16287** — the final runs kill #10, #16 and ALL THREE roof guards and die at (197.95, 10.56,
  −278.88) in the annex descent, 86.4 of it to #32 (124 hits at the 180-tick cadence — the
  ground spawner that spawns during the climb window and shoots from (158.9, 1.0, −229.2) at
  d 56-92) plus 13.6 fall damage. The twin is still `#[test]` without `#[ignore]` — it is
  red-with-instrumentation by design until the last two shooters come off the table, and every
  assertion in it is [M]/[D]/[L]-cited from the prior reports; the new kill legs are LOGS, not
  asserts (the kill_guard_at contract).
- `probe_0d_ray_stands` (new, assertion-free): run 3 times, the matrix in this report and the
  test comments.
- **nf-game, enemy_accuracy_live and the other walkreal binaries: NOT RERUN this session** —
  no `crates/*/src/` file changed (git shows the session touching only
  `crates/nf-view/tests/walkreal_0700000d_live.rs`, the new `probe_0d_ray_stands.rs`, WORKLOG
  and this report), so those batteries' code paths are byte-identical to the merged master
  state; the shared test helpers were not touched (lesson 7's rule). The suite runner was
  started in the background twice (17:55, after the origin/master merge) and was killed between
  tool calls both times — the same measured behaviour the -1/-2 reports document; no
  `suite.out` total line exists. THE FULL SUITE ON THE FINAL TREE IS OWED — run it before
  merging this branch.

## What's left (the next message's levers, in order of promise)

1. **#32's kill stand** (the last big drain, 86.4): its rounds die on a ~1.7-high ground
   obstacle at (160, −217) when fired from the strip mouth (Bond's eye 1.7 clips it —
   `[0d:ray] full 0.960@(1.0,0.0,0.0) bullet_scoped 0.960`, two probes agree) while #32's own
   rounds clear from its higher eye (0-damage return hits measured at d 56). Levers: an
   east-approach stand (the ray into the patrol band from x > 175 at ground level), the
   warehouse-floor stands (y 12.1 — the -1 measured the dock face eats steep shots from the
   west, but #32 is NORTH-EAST of the placements, a different angle), or the trench rows. The
   patrol band is measured: (157.59, −213.01) → (158.9, −229.2) → (163.3, −214.0).
2. **#18** (its one ride hit, 16, plus bay-phase fire): the in-bay fight needs the guard's
   LIVE patrol spot at fight time (it was measured at (207.26, −288.74) and (211.96, −288.73);
   the (210, −288.8) ±(8, 5) box found nothing — widen to the whole band or read the box dump
   first: the `[0d:box]` line prints every live guard within 14 u before the pick).
3. **The 13.6 "other"**: fall damage on the late route (the annex drop chain) — measure which
   leg (the death summary's `other` bucket) and soften the landing legs.
4. The full suite + every `walkreal_*_live` binary on the final tree (lesson 7) before merge.
5. The live checks from the -2 report stand (its checks 1-3, unanswered).

## Live checks for the main agent

1. **#32's spawn and post**: does the REAL game's #32 (skin 0x5000093, the ground band
   (157.6..163.3, −213..−229), weapon 0x1e) fire on a player in the bay/yard at d 92 with the
   roofs dead? gdbwatch on its state byte — if the real #32 holds fire at that range, the port
   over-acquires (its sight_range at the census was not printed — print it).
2. **The ground obstacle at (160, −217)**: what is it? A [D] coll-scan column at (160, −216..−218)
   for the 0..2 y band — if it is a creepwall or an NPC-only blocker, the port's PLAYER bullet
   path may be eating a blocker the game's bullets ignore (the -2 proved both paths share
   `segment_bullet_scoped` [M 0x1ea398:216-229], so a divergence would be a real find).
3. The -2 report's checks 1-3 (the #10 ladder-top acquisition, the deck-arrival asymmetry, the
   engage→reposition law) — still open.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).
