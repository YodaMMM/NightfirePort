# WORKLOG

### level-11-stairwell-bullet-holes-6 (started 08:24 UTC, 2026-10-29)

Task: land the scoped flush re-query the -5 report describes
(`segment_surface_flags_scoped(cell, from, to)` mirroring `segment_bullet_scoped`, the flush
using `find_cel(cur)`; [M] `Bullet_CollisionHandler` 0x12ce80's HITDATA walk) and re-measure
the stairwell fan. If the one visible mid-air decal at (23.20, 28.22, 5.77) remains, compare
the flush's per-hit processing line by line against 0x12ce80 (post-ricochet scoped pick; the
glist-0x10 node rule in `Collide_Intersect` 0x1ebaa8) and fix the remaining difference, or
prove it game-true [M/D] and pin that. Un-ignore the pin when green. Full suite, quote the
total line.

Plan: branch pushed first; downloads + warm build next; suite runner backgrounded in the
first minutes; then the scoped query lands, the fan dump re-measures, and the per-hit walk
gets the line-by-line read.
