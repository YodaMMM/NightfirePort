# WORKLOG

### level-0c-east-climb (start, 04:47 UTC, 2026-10-31)

Task: `walkreal_0700000c_live` still places Bond at `[95.0, 11.3, -209.5]` for the east
climb: staircase01's bottom floats 9.52 over the 6.35 under-stairs [D eastclimb3/4], the
jump apex is 0.78, no grapple point serves it (the five `graple_pulley` points sit
elsewhere) and the only ladders are #164 and #495 [D]. Find the game's route to that
platform [D the placements, movers, script entities that pose collision (a lowered stair,
a lift), the level's objective order — maybe the route reaches it from above; M any mover
class], walk it on real input and delete the write; or, if the data shows no route, pin
that with a sister assert and name the live check that settles it.

### level-0c-east-climb (finish, 05:50 UTC, 2026-10-31)

The east-climb band deck PINNED a collision island; no route exists in the disc's data, and no
mover serves it. The evidence chain (all [D] via the port's parsers, this session's committed
probes eastclimb5/6/7): the turb4 stair flights are horizontally DISJOINT — staircase02 (plc36)
tops at 8.10 at x 81.3..81.4 and staircase01 (plc35) starts at 9.45 at x 83.8..84.4, with the slot
x 81.5..83.7 EMPTY from 6.5 down to the lower flight's 1.75..3.75 (the fine map); the band strip
gaps 10.4 m over the chasm (the west band ends 56.08, turb3_walkway04's deck starts 66.5); the
north-strip mezzanine (10.85, z −206..−205.1) breaks at x 54..58.5 (room02's 10.17 sliver
58.9..63.3 is itself an island); crates4 (tops 7.31/8.31) sits 3.1 m south of walkway02's dead
south end 10.37 with a 2.06 rise. The flood-fill over the sim's own collision (0.25 grid, directed
edges, drops one-way) leaves the gate deck + staircase01 deck one component UNREACHABLE from the
east ground, the yard, the west band, the corridor, the gallery and the catwalks — at step law 0.55
AND an absurd 0.80. No mover: no lift kind on 0c [D kind census], the 8 SSs / 2 doors / 2
ScriptPlayers are west-side scenery [D audit §6], ladders #164/#495 and all five graple_pulley
points west/north [D]. The game's own nav patrols the east band (G3, 10.34) and east ground (G4,
6.17) as SEPARATE paths with no links [D nav dump]. Sisters 5b/5c added to walkreal_0700000c_live
(the slot void + the three remaining crossing attempts); the stand-in stays, documented. Touched
binaries: walkreal_0700000c_live 2 passed / 0 failed (18.8 s), interactables_0700000c_live 5/0,
walkreal_0700000c_eastroute_live 1/0; the walkreal_0700* mission sweep 25 binaries all green
(01 5/0, 02 1/0, 03 4/0, 04 7/0, 05 10/0, 06 3/0, 07 2/0, 08 3/0, 09 4/0, 0a 7/0/2i, 0b 3/0, 0d
1/0, 11 1/0, 11_route 1/0, 12 1/0, 13 1/0, 14 6/0, 15 1/0, 16 8/0/1i, 1b 9/0/1i, 41 5/0, 4a 1/0,
4a_satchel 2/0 — totals quoted in the report). The full-suite total line: NOT run this session
(the background fast runner died at a tool-call boundary again; the foreground window went to the
mission sweep). Report: docs/reports/level-0c-east-climb.md.
