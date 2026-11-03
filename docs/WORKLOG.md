# WORKLOG

## level-41-exit-event-2
Started 2026-11-03 06:20 UTC. Branch s/level-41-exit-event-2 cut from origin/master 6f16146e.
Task: on master, `interactables_07000041_live::the_wrapper_ends_into_the_space_station_after_the_chain`
is red since level-41-exit-event landed (the 241-tick wrapper-end stand-in removed, the 0x18→0x1b
resolve added). Find how the game's 41 exit event and the wrapper's end-into-the-space-station
chain coexist [D the 41 scripts/channels; M the level-exit / LOAD_LEVEL path], fix the port, keep
the wrapper test green without weakening an assertion. Run every *07000041* binary, the
campaign/hand-off binaries and nf-game.

Work log:
- (06:20) branch start, WORKLOG entry. Next: fast runner in the background, then the red test.
