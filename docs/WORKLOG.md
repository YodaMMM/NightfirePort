# WORKLOG

## level-0c-every-objective-blockers — 20:07 UTC 2026-10-29
Branch from origin/master (02e306a6). Task: 0c (Chain Reaction Part 1) blockers 2-5 of the
level-0c-every-objective report, in order: (2) plate #274's cell inside the display case
(glass body-block? or the plate fires another way), (3) the display ground -> warehouse/yard
descent, (4) the ring's west/south gaps (south = the #283 wire ride onto the west ledge),
(5) the doorway-ledge -> yard descent. Each from the game's behaviour [M/D], fix port gaps,
replace each stand-in in walkreal_0700000c_live with the real-input leg. FULL suite, quote
the total line. Deadline ~22:03 UTC, stop new work 21:43 UTC.

## level-0c-every-objective-blockers — 21:50 UTC 2026-10-29
Three probe binaries committed (walkreal_probe0c_blk/blk2/blk3_live) mapping all four stand-in
regions [D]. Findings: (2) plate #274 hangs in the walkway's north TRENCH (x 5..6.5, z −110.5..−111.5,
floor 11.84 = the lower walkway's north end) between the case's glass walls; the south glass blocks
the walk-in at body height — the game's entry is the four-hinge laser burn (#139-142 → AND #505 →
ch 18 → the #138 grate 16.4 → 14.76). (3) The display→yard descent EXISTS [D blk3]: slab south edge
→ deck 2.37 → yard −0.02 — the old "void at z −154" was a y 9→1 ray missing the sub-1 yard floor;
the annex vent (lock #180 out 0x54 + the SS lid 5.43→0.0) is the alternative. (4) The gaps' crossings
are the WIRES: ctx-3 icons (#4/#6/#7/#12) mark three type-0 wires whose derived grab segments span
both gaps and land on ledges/corners [D]; the ride mechanics are ported [M]. (5) The ledge south wall
spans x ≤ 55.5 only — x ≥ 56 is open (floor 0.90 → 0.69) [D rays]. The real-input legs were written
(the trench walk-in, the slab→deck→yard→plates walk, the ledge east-strip descent, three wire
rides) but the session's validation window closed before they ran green: leg 1 wedged on the case
south glass (the hinge-burn entry stands), leg 4's approach needs the slab reachable on real input,
leg 5b wedged at x 55.0 on an unprobed face, the wire rides unvalidated. The walkreal was reverted to
the master stand-in shapes (GREEN 2/0) with all findings inline; the legs' code is quoted in the
report. Probes + walkreal comments pushed. Suite: the walkreal binary 2/0 + the three probes green;
the FULL suite did not fit the window (sandbox expiry ~22:03).

## level-0c-every-objective-blockers-2 — 00:06 UTC 2026-10-30
Branch from origin/s/level-0c-every-objective-blockers (a0cdf6dc WIP), merged origin/master
(21fbeed3: the cable third-person work + the scoped bullet re-query). Task: the merged
walk_the_whole_mission_from_the_start_to_the_0d_handoff goes RED at the #283 ride — "must drop
onto the ring SE corner (11.09..11.17), got [35.15, 13.55, -209.95] dropped true"
(walkreal_0700000c_live.rs:454). Find what moved the drop (the face Bond stands on at y 13.55,
the drop timing, the wire's end) [D/M], fix the port or re-derive the leg, keep the walk green.
FULL suite on the final tree; quote the total line. Deadline ~02:02 UTC, stop 01:42 UTC.

## level-0c-every-objective-blockers-2 — 01:00 UTC (final for this window)

The #283 ride's drop was NOT moved by the merge — the leg was never validated (the WIP said so)
and its drop timing was wrong. Evidence: the corner floor's west edge is x ≈ 35.8 (a 10.89/10.69
step at z −210..−209.8, the 11.06..11.09 corner east of it [D ray maps]; the 11.93 lip at
(35.8, −209.6..−208.8)), while the old `jump at d < 0.9` fired the FIRST tick inside 0.9 of B
(35.89, −209.44) — at (35.15, −209.95), 0.65 WEST of the edge, where the port has NO face at all
(y 15..8 under the point: empty [D rays]) — Bond fell to the yard (root 1.41 in the probe). The
RED's [35.15, 13.55, −209.95] is the HANG pose (wire y 14.44 − the 1.0207 hang offset [M
wire_tick 0x1976dc's `hang`]), snapshotted because the settle loop's `if grounded break` breaks
on the exit tick's STALE grounded flag — wire_tick's drop path zeroes vel and returns; the flag
clears the next tick [M 0x197758's exit arm]. The ride itself reaches past B: the strafe zero
rule parks the input at d_B ≈ 0.5 [M 0x1977c0] but the playing mode-7 move anim finishes its
root motion and overshoots to d_B ≈ 1.1 PAST B — (36.70, 13.41, −208.66), over the 11.06 corner
floor [probe trace]. Fix (test-only): ride to the natural stop (d no longer shrinking for 90
ticks) and jump there; settle waits air-then-ground. The landing [37.28, 12.10, −208.45] —
root 12.10 on the 11.06 corner — inside every [D]-backed band of the existing assert, which is
unchanged. walkreal_0700000c_live 2 passed / 0 failed (the whole mission 6366 ticks, stuck 0,
falls 0; the 0d handoff fires objective 3). No port code changed — the merge brought no
player.rs/world.rs edits (02e306a6..21fbeed3 touches drone/lock/gadget/nis/progress/stats only).

Suite: the fast runner (tools/test/suite.py) launched 00:05 UTC, build ok 923 binaries in
1168 s, sweep running on the final tree (the 0c binary re-linked with the fix before its turn);
the sweep cannot finish inside this window — the total line is reported as far as it got.

## level-0c-real-crossings-2 (started 03:05 UTC 2026-10-30)

Task: blockers-2 (merging now) made the whole 0c walk green with the #283 ride re-derived.
origin/s/level-0c-real-crossings made the hinge-burn leg REAL (the hinge locks burned through
held fire, the trench drop fires plate #274) but its #283 landing went red. Port the
real-crossings hinge-burn leg onto master+blockers-2, then the remaining wire crossings
(#284, #282) and descents on real input, keeping blockers-2's #283 leg. The walk stays green.
FULL suite, quote its total line.

Plan: branch from origin/master + merge blockers-2 (done, fast-forward 6082a67a); then port
3b52d8e8's hinge-burn leg + ab29c2ba's #284/#282 rides + 29b43fa9's FIRE-action wire drops;
run the walk green; suite; report.
