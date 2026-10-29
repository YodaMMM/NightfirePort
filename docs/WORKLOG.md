
## level-0c-door2turbine-2 (started 05:42 UTC 2026-10-29)

Task: Chain Reaction Part 1 — walk to the door2turbine E stance for real on 0700000c. The
level-0c-door2turbine report leaves the walker balancing on an unexplained ~13.05 surface at
x 40.45..40.6 between the SE walkway and the 11.61 shelf / 11.09 corridor (owner not in the
static census): find it [D] (drone, dynamic object, or missed collision mesh), fix the port
collision gap [M/D], then replace the stand-in with the real-input leg (corner E opens #281,
the slot jump onto (44,-209), the band walk), then blockers 2-5 of level-0c-every-objective
in order. Full suite, quote its total line.

Plan: branch pushed from origin/master (f1791382), fast-suite runner started in the
background now (master moved past the warm cache, so the build phase recompiles first),
then read the level-0c-door2turbine and level-0c-every-objective reports and reproduce the
balance on real input.

Progress (06:40 UTC): the 13.05 mystery solved — no such surface exists; it was Bond's body
centre on the 12.02 slot floor [D probe]. The district re-mapped: the SE walkway is two
ceilinged dead-end boxes, the real route is the west ledge 11.61 (z −222..−213) → the 0.52
drop onto the 11.09 corridor → door2turbine #281. The real-input leg now runs: ledge →
corridor → the REAL E press opens the door (state OPEN, ch_open 5 on [M 0x135be0/0x135618])
→ the corner → the doorway tunnel in the east wall → the slot jump LANDED on the band
(44.84, 12.18, −208.80). One wedge left: the band x 53..67 has a 12-unit floor gap [D band
west map] — the next session's item, with blockers 2-5. Report at
docs/reports/level-0c-door2turbine-2.md; probe walkreal_probe0c_door2_live committed.

## level-0c-door2turbine-3 (started 07:50 UTC 2026-10-29)

Task: continue level-0c-door2turbine-2 — its walk is real up to the band east, then RED:
the band floor has a void at x 55..65 (z -209.5..-206.0) [D its probe]. (1) Find how the
game gets from the band (x 44..53, floor 10.34) to x 67+ [D disc around x 53..69,
z -209..-195: the 10.85 strip at z -206..-205.2, the descending east walkway x 61..69,
z -204..-202; any trigger, door, lift or script that bridges it]; fix the port if the
world lacks a disc connection [M/D]. (2) Make the walk green through that crossing on real
input. (3) Then blockers 2-5 of the level-0c-every-objective report in order. walkreal green
before push; FULL suite total line quoted; report at docs/reports/level-0c-door2turbine-3.md.

Plan: branch cut from origin/s/level-0c-door2turbine-2 (61855c02), origin/master merged
clean (3-file props/walkreal conflict-free), pushed now. Next: p7zip, work-slim + ISO
downloads, read the -2 report and the level-0c-every-objective report, reproduce the band
RED on real input, probe the disc for the crossing.
