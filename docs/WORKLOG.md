# WORKLOG

## level-14-west-door (2026-10-28, branch s/level-14-west-door)

Task: 07000014 (Countdown Part 1) — walkreal_07000014_live
`walk_the_infiltration_to_the_delta_exit` wedges at the facility's west double
door at (-52.2, 1.2, 40.1): two door records, kind 2, state 0x2, pos01 0.00,
ch_open 0, ch_lock 0x85, lock_flag 0. Find how the game opens it ([D] the
placements writing channel 0x85 and their triggers; [M] the door's lock/unlock
and use paths, Door_Update / the lock channel read) — a switch, an objective
step, a script, or an approach side the port lacks — and port the missing
link. Then walk legs 2..6 of the test on real input to the Delta exit #944 and
un-ignore it (or re-ignore on the next exact blocker with its evidence). Full
suite before push, quote the total line.

Plan:
- Start the fast runner in the background in the first minutes.
- Read the wedged test + door records; census the 07000014 placements for
  channel 0x85 writers and their triggers [D].
- Read Door_Update / lock channel read / use paths in work/decomp [M with asm
  where the C looks incomplete].
- Port the missing opener; write real-input tests; un-ignore legs 2..6.
- Full suite (cargo test crates + fast runner), push, report, finish.
