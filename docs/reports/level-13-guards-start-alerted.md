# Report: level-13-guards-start-alerted

Task: Phoenix Fire Part 4 (level 07000013) lobby guards start alerted at level start, before Bond has been
seen. Find each guard's start state on the game's path ([M] Drone_Create / the drone's initial AI state and
alert fields, the level's start scripts and channels that may alert them, any SP/script that sets the alarm at
load) and [D] the placements' AI params, then port what the game does so they start idle/patrolling and only
alert through the game's own detection.

## Plan (WORKLOG start entry)

1. Branch `s/level-13-guards-start-alerted` from origin/master (6ff39ea9) — done.
2. Background suite started (fast runner, tools/test/suite.py).
3. Locate the port's level-13 / guard spawn path and the current guard start-state code.
4. Read the decompile: Drone_Create, the drone's initial AI state/alert fields; level 07000013's start
   scripts and channels; any SP/script that sets the alarm at load.
5. Check [D] placements' AI params for the lobby guards.
6. Fix the port to match the game's rule; test from a fresh level load on real input (tick 0 and after N
   ticks with Bond hidden; then the first sighting alerts them).
7. Full suite, checkpoint pushes every 20-30 min, final report.

## What's done

- (in progress)

## How to verify

- (filled at the end)

## What's left

- (filled at the end)
