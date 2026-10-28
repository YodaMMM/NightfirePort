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
