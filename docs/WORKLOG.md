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
