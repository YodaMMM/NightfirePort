# WORKLOG

### level-11-stairwell-bullet-holes-5 (started 00:12 UTC, 2026-10-29)

Task: 07000011 (Phoenix Fire Part 1) stairwell bullet impacts — reports of bullet holes
hanging in mid-air along the railing level, on a plane in the well and looking up, and
invisible walls eating shots (10-25; 10-26 "maybe progress?" unresolved). The per-caller
bullet/LOS masks [M Collide_Intersect 0x1ebaa8:300-319] are being finished on
s/capsule-cull-and-los-masks-11. FIRST measure the current state on master AND on that
branch: fire from the stairwell spots across, up and down the well, record every impact
whose point is not on a drawn surface (impact's collision face vs the render geometry
there). Only if mid-air impacts remain: find which collision faces stop the port's bullet
that the game's bullet passes [M Bullet ray mask, Collide_FilterBullets 0x1ece30, the
surface byte drops] and fix the bullet path. Report the measurement either way. Full suite,
quote its total line.

Plan: push this branch first, start tools/test/suite.py in the background, then build a
stairwell bullet-impact measurement test (real fire path) and run it on master and on
s/capsule-cull-and-los-masks-11; compare impact faces against render geometry; fix only
what the measurement proves broken.

## level-11-stairwell-bullet-holes-5 (2026-10-29)

Measured 07000011's stairwell bullet impacts vs the render geometry on master AND on
s/capsule-cull-and-los-masks-11 with a new fan test (4 spots x 8 yaws x 5 pitches through the
real input path; impacts + decals measured against the drawn World triangles). Identical
numbers on both trees: 101 world impacts / 5 mid-air impact records (all on mat-0x40
pass-flagged faces — invisible: mat&0x3f==0 blocks Effect_Create [M 0x1ce6f8], no render
consumer for player_impacts) and 1 VISIBLE mid-air decal at (23.20, 28.22, 5.77), 0.40 from
the nearest drawn triangle, from the frame2944 spot yaw +0.79 pitch -0.50 ricochet chain —
the 10-25 symptom reproduced [L]. Tried the [M]-grounded scoped flush re-query
(Collide_Pick 0x1eb320:125's StraddleCels-only candidates): the numbers did not move — the
faces are IN the straddle set — so the surviving hole may be game-true (undrawn collision
planes with real material bytes in this declared-fudge stairwell); needs the main agent's
render check (report's "Live checks"). The scoped re-query implementation is described in the
report for the next session to land after a full suite (RNG-stream blast radius). The pin
runs ignored-RED with the numbers. FULL SUITE NOT RUN: this sandbox's 10 GB disk filled twice
during the rebuild churn (ld signal 7 Bus error); pruned 3.9 GB of stale test executables but
the window did not fit the relink+run. Next session: the live check decides the rule; the
scoped flush lands after a full suite.
