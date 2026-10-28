# WORKLOG

## level-0b-every-objective (2026-10-28, branch s/level-0b-every-objective)

Task: make crates/nf-view/tests/walkreal_0700000b_live.rs walk the WHOLE 0700000b
mission from a fresh load on real input (step_bond only; no teleports past a
blocker, no channel or state pokes): every objective the level's mission data
lists [D objective records / Mission_MonitorObjectives 0x1827d8 M], in the order
a player does them, each asserted complete, ending on the level exit firing.
First list which objectives the current tests already cover; add legs for the
rest. Every blocker is a port gap: find the game's behaviour [M/D] and fix it,
or — if drawing only — list it for the main agent. Full suite; report; finish
steps.

Plan:
- Fast runner started in the background in the first minutes.
- [D] the level's objective records (0700000b.bin) + [M] Mission_MonitorObjectives
  0x1827d8: the objective list and completion channels.
- Survey walkreal_0700000b_live.rs: which legs exist, which objectives covered.
- Add the missing legs on real input; fix the port gaps the blockers expose.
- Full suite on the final tree; push; report; finish steps.

## level-0b-every-objective-2 (2026-10-28)

Branch `s/level-0b-every-objective-2` from `origin/s/level-0b-every-objective`
(28c84851) merged with `origin/master` (ad2f0791).

Task: continue 0700000b end-to-end. The rewritten walkthrough fails at the
Q-Worm install leg ("the Q-Worm C install never landed" [M 0x1aee20]) though
the pre-shot state matches the merged version - bisect the legs before the
shot (walker stuck-mitigation side effects in the wedged terminal/lock walks,
the exit-request state vs the bullet pass) and fix the real cause. Then the
lift boarding (cab_y never matched hash 2000367 - pair the entity id/hash with
one probe) so the balcony -> exec floor transit is walked, and the walked lock
and terminal approaches past their pinned furniture blockers [D probes] - each
from the game's behaviour [M/D]. Real input only. Full suite; report; finish
steps.

Plan:
- Fast runner started in the background in the first minutes.
- Reproduce the Q-Worm install failure on the current tree; bisect the pre-shot
  legs (wedged terminal/lock walks' stuck-mitigation side effects, exit-request
  state vs the bullet pass).
- [M 0x1aee20] the install gate; [M/D] the lift entity (hash 2000367) and its
  cab_y response; one probe to pair id/hash.
- Walk the lock and terminal approaches past their pinned furniture blockers.
- Full suite on the final tree; push; report; finish steps.

Results (level-0b-every-objective-2, continued):
- The Q-Worm install failure's real cause was NOT the shot path: the rewrite's shaft-mouth wait
  slid/hopped Bond into the paternoster shaft and his capsule crossed the y 7.98..11.98
  out-of-bounds slabs (trig40-45, kind 2 out 0x7A [D boxes]) -> OR #46 inputs [0x79,0x7A] ->
  ch 0x7B -> the fail wedge 0600098A played (its event 4 disables Bond; no re-enable on this
  path) — step_bond's disabled gate then ate the shot's trigger forever
  (`let (trigger,..) = if self.disabled {..(false,false)}`), so no 0x58 bullet ever spawned
  (`bullet_paths 0` after the whole shot window) and `PCQWorm_Update` 0x1aee20 never saw the
  class-5 toucher. Bisected with in-test probes: disabled false after the shaft walk, ->true at
  boarding-poll +19, ch7b true at the flip, mission_status 3 by the pre-shot.
- The boarding poll was blind: `cab_y` compared the DECIMAL literal 2000367; the runtime script
  entity's hash is 0x2000367 = 33555303 (the probe dumps the pairing: hash 0x2000367 kind 49,
  matrix x/z LOCAL (0,0), only [3][1] carries the keyed y; the collision box is placement-anchored
  x 0, z 23.19..26.56 [D placement #232]). Fixed to 0x2000367.
- The ride is now REAL and the walkthrough completes end-to-end (24136 ticks): the wait spot is
  the balcony's proven z~25.8 corridor 0.7 east of the lip with the walk's residual velocity
  zeroed; the board is the player's centred hop during the 499-tick bottom hold (the cab's stand
  surface is cy + 4.1 = 15.1 at key 0 [L lifts_ride_live grounded 16.11] — a landing near the box's
  east face gets shoved off by the 0.06/tick rise, so the board targets dxz < 1.0 && x < 1.0);
  the carry [M 0x194188 -> Control_InheritVelocity 0x133e88] rides Bond to the top (trace: y
  16.13 -> 22.77 -> 36.14 -> 48.60 with the cab), the step-off walks east off the level surface.
- Attempted (not landed): the walked desk/terminal/lock approaches through the nav graph —
  walk_nav_to's wedged wandering (3 x 6000-tick caps) re-trips the dome-hole fail web (disabled
  true at the pre-shot again), so the tree keeps the checkpoint's documented stand-ins; the nav
  routes on the exec floor need validating before the walks can run (left item).
- Suite: the sandbox kills background jobs between tool calls (the fast runner died twice with an
  empty suite.out, confirmed again this session) and the foreground window does not fit the full
  four-crate pass; the touched test binary ran green (2 passed) and the final-tree slice is in
  the report.
