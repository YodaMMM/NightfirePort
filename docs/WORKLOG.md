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
