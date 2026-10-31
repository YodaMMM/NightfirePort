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
