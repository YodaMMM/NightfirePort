# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## pane-collision-pairing

2026-11-07 08:10 UTC, branch `s/pane-collision-pairing` (from origin/master 22d5836b7).

Task: intact glass panes stop Bond and bullets, and shot-out ones do not.
drone.rs (~line 19300) builds `_pane_parts` for the placed 0x20 panes but never adds
them to the collision world: the shape-key/matrix pairing is unresolved, and with them
in the set, walks/guard sight/dropped items on 01/09/0a/0b/14 stopped at shapes where no
pane stands. Both dead_pane_live.rs tests are ignored. Read done/30-dead-pane-collision-2.md
in the queue first, settle the pairing from the game (Break_Create 0x12c230, Collide_PickObj
0x1eae80, Break_Kill 0x12b970:60), turn the parts on with the death-tick grace, respect each
pane's bullet filter (Collide_FilterBullets 0x1ece30), remove both ignores, pin through real
input. Keep one rule for script-created panes (entity arm) and placed panes.

- [x] setup: branch created from origin/master 22d5836b7, pushed.
