# WORKLOG

## real-damage-routes-08-11-14-03

- **Branch start** (sandbox5, resumed 2026-11-01 ~11:00Z): the claim's walker-side work was in the
  tree unpushed (no remote branch existed). Committed as the first checkpoint: real-damage wrappers
  for 03/08/11/14 (`walker::with_real_damage(...)`), the captor spares for 03, the shooter re-fight
  rule (a guard still shooting Bond is re-fought after 120 ticks instead of the 240 shield), the dry
  resupply after a fight, answer-shooters-during-weapon-cycle, and lenient restoration in
  `try_walk_to`/`walk_nav_to`. Next: build, run the four walkreal variants, iterate.
- **Outcome** (2026-11-01 ~12:20Z): 03 real-damage PASSES (82.2 s). 11's stair-height break fixed
  (the flight C/B legs now target past the flight bottoms — a radius-1.0 arrival can no longer
  stop 0.013 over the band); the walk descends, loads 07000012 and dies later in the east wing
  (tick 3122, shotgun volleys — beyond the claim's break). 08's walkway wedge is gone (the walk
  boards and rides the gondola) but Bond dies at the Hind's opening blasts (74.2 damage = exactly
  the arrival health after the #19/#20 shaft fights cost 25.8). 14's west pit jump still lands on
  the stairwell frame (the landing is two narrow strips beside a pit gap [D probe grid]) — the
  edge-anchored attempts wedged on a phantom lip 0.39 u from the anchor; reverted to the committed
  route. fast_taps (auto rip) tried and REVERTED — it broke 03's fight timelines. nf-game 1014
  passed / 0 failed / 9 ignored. Left: 08 blast economics, 11 east wing, 14 west jump, the
  origin/master merge + the final suite run.
