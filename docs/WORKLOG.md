# WORKLOG

## level-1b-scripted-carry (branch s/level-1b-scripted-carry)

Start: 2026-10-30 12:31 UTC, branch from origin/master 9deb3338.

Task: walkreal_0700001b_live::walk_arrival_to_drakes_defeat is ignored: the west wall plc #128
OBJECT_Boundary_1 (x 4.433) is body-solid [M], so the game's crossing must be scripted. The
candidate is the SP#211 chain (0x060007fd -> 0x060008f0): it pulses ch 75 (the float entry) and
its tail pulses ch 74 (0x600096e, t392-399 [D]). Decode those scripts fully [D the streams;
M the script player/event handlers 0x1d94c0, 0x1d8548]: find any player-position write, attach
or teleport event, and which entity the player follows. Port what is missing, then make the walk
cross on real input (the script doing the carry) and un-ignore it. Every test in the file green.
