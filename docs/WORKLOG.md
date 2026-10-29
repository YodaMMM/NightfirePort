# WORKLOG

### level-14-mezzanine-descent (start, 23:33 UTC, 2026-10-29)

Task: Countdown Part 1 (07000014) — the walk from the vent scene down into the facility. The
vent cut-scene 0x06000833 now hands Bond back where he stood [M 0x1d94c0 case 6; L PCSX2 fresh
07000014: touch #301 at (40.7, 3.1, 39.0), stands at (40.68, 2.76, 38.99)]. The old route of
`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` rode the removed teleport;
the mezzanine descent wedges at (32.06, 2.70, 39.4) — the wall at x 31.5, z 38.4..40.8, and the
floor edge south of z 38.4. Find the route a player takes from that spot into the facility
[D the collision and placements around the mezzanine: stairs, ladders, drops, doors, vents;
M any use/jump path], fix any port collision gap, walk it on real input, un-ignore the test
with the rest of the walk green. FULL suite; quote its total line.

Base: origin/master (48696227). No prior work on this branch (continuation message; the branch
had zero commits — this entry is the first). Suite runner launched at 23:31 UTC
(tools/test/suite.py, suite14.out).

Plan: read the level-14-skip-handback report + the walkreal ignore reason; map the mezzanine
collision [D] + any use/jump path [M]; port collision fixes with game addresses; walk on real
input; un-ignore; full suite on the final tree.
