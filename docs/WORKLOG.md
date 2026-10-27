# WORKLOG

One entry per task, appended at the end. Never rewrite or drop another task's entry.

## level-01-mayhew-wait

Task: on 07000001 (Double Cross Part 1) the Mayhew escort runs far ahead of Bond and goes
through doors before him; in the game the escort waits at its hold points. Find the escort
wait/hold rule in the decompile (escort follow states + player-distance gates, door-wait
logic; `escort_follow_07000001_live` and `fix_npc_doors_live` pin parts of it), the route
points on the corridor [D], fix what lets him run ahead, pin with a real-input walk test
(Bond stays back, Mayhew holds at his wait point).

Status: started 2026-10-27 ~00:52 UTC. Setup: fresh clone (sandbox reset), branch
`s/level-01-mayhew-wait` created from origin/master @ ec917093 and pushed.

### level-01-mayhew-wait — outcome (2026-10-27 ~02:20 UTC)

The ally-wait hold is implemented (Drones::tick parks the escort in 0x21 at the 0x20 node until
ch 0x99) and the release chain is proven end-to-end through the real input path: plate 0x32 →
OUT mux 0x33 → SP #276 → 06000086 → ch 0x0e → the three ambush guards → death channels →
AND mux → 0x99 → the hold releases. Green: mayhew_hold_live (new), escort_follow_07000001_live,
fix_npc_doors_live (its door-#1 premise fixed per the hold; the escort's first self-opened door
is #2). The old "ch 0x21 writer unknown" note was a mis-read of SP #276's out param (0x33).
Left: the full suite on the merged tree (this sandbox kills background processes; the suite
needs ~40 min of foreground windows), and walkreal_07000001_live may need the same
plate-crossing harness step. Report: docs/reports/level-01-mayhew-wait.md.
