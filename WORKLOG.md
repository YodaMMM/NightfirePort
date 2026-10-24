# WORKLOG

## 994-zoe-gondola-real-path (2026-10-23T10:24:08Z)
- Claimed; setup: clone done. Plan: real-path test on 07000008 gondola (Zoe targeting + riding), then fix port.
---
## 994-zoe-gondola-real-path (2026-10-23T12:02:16Z)
- Test + riding port landed; the team-2 opponent model port was attempted and REVERTED (the 07000001 matrix stream breaks; the evidence is in the report). (a) stays red pending 957b; (b) green.
---
## level-05-west-lane-3 (2026-10-23T21:41:00Z)
- Claimed; branch s/level-05-west-lane-3 created from origin/master (8b101249d). Plan: settle the two route-map disagreement (master's tower-top walk-off vs s/level-05-west-lane-2's castle-wall lane), then the west lane on the real walk: the z-92 lip, creepwall #299 crawl, lane-to-ledge link, the cable notch grab, the ride, window #35 crawl, exit.
---
## level-05-west-lane-3 (2026-10-23T23:22:30Z)
- The two route maps settled against the disc collision: the castle-lane map (west-lane-2) wins; master's roofs-west continuation is an isolated y-28 island (no walk edges) [D the connectivity census]. The lip at x 42-44 is NOT cleared by the game's step rule [M r-0.55 sphere; no step assist in SSWalk/Move] — the sidestep columns walk green. The west lane (strip → castle lane → tower top) is UN-IGNORED and passes. Port fix: the creepwall half-length now reads the disc's object bounds ([D] Short x −3.62..+7.52, Window ±5.32; the old [S] scale.x=1.0 exited over the crawl band's dip). Two round-4 blockers named with walked stalls: the tower-top west parapet (37.1 kerb at x 26) and the crawl shelf wall at x ≈ 26.8. Report: docs/reports/level-05-west-lane-3.md (live checks listed).
---
## level-05-west-lane-4 (2026-10-23T23:47Z)
- Claimed; setup: repo + work/ + ISO already present. Branch created from origin/master, merged s/level-05-west-lane-3 (the west-lane walker + creepwall bounds fix were not yet on master). Plan: map the parapet/shelf (x 24-27, z 58-76) at 0.25 res, find the real plateau→shelf route, then the crawl/slab-end/notch/ride/window-ledge exit leg; settle creepwall +0x8c from control_create_object; un-ignore walk_the_west_ledge_and_cross_the_cable.
---
## level-05-west-lane-4 (2026-10-24T00:55Z)
- The two round-3 blockers answered by the 0.25-unit map: the tower top's SOUTH edge has no parapet (the slope 36.0→33.4 wraps west onto the south ledge), and the crawl shelf is reached by that same ledge. The crawl legs walked UN-IGNORED and green (the crawl #299 entered at the live spot, exited north across the waist after 513 ticks, the slab east end reached). The creepwall +0x8c settled [M Control_BuildWorldSph 0x133070]: bound radius × scale, the centre = the world-sphere centre (the offset rides). The slab crossing stays parked: the east tip is fragmented [D] and the one-contact rework unfounds walk_to on the slab [L]. Report: docs/reports/level-05-west-lane-4.md. The full suite was restarted on the final tree at 00:48 UTC; its total line is quoted in the report (the round's first suite start died silently).
- Suite correction (00:48→01:10 UTC): both background suite starts died silently between tool calls (the sandbox kills background jobs); the foreground chunks covered the final tree instead — nf-game 86/86 binaries green, the 6 creepwall-consuming nf-view targets green (53 tests), the 07000005 set green (walkreal 4 passed/2 ignored, interact 11, walk 2). nf-formats/nf-audio are byte-identical to master on this branch.
