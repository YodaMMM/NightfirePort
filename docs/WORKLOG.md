# WORKLOG

## level-41-every-objective
Started 2026-11-03 04:10 UTC. Branch s/level-41-every-objective cut from origin/master
6f16146e. Task: no walk pins every 07000041 objective in order — extend walkreal_07000041_live
or add a sister test that plays the whole mission on real input from the fresh start, every
objective completing in the game's order, each checked on the channel or objective state the
game reads (objective records, channels, scripts [D the level file, M the objective handlers]),
not on Bond's position. Bond reaches the level end alive, no teleports, no forced fields. Fix
the port against the decompile where a step can't be done the way a player does it, and fix the
whole class. Stay off the exit event (pending 30-level-41-exit-event-2) and name it in the
report if it blocks. Run walkreal_07000041_live, interactables_07000041_live and nf-game.

Work log:
- (04:10) branch start, WORKLOG entry; fast suite runner next.
