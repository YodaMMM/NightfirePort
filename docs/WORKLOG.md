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
