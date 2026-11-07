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

- [x] the pairing settled from the game: Break_Create 0x12c230 takes the caller's glist
      (Control_SetGList 0x134960; the placement path passes pCelList[index] / the hash hit,
      NULL and 0xa000-flag rows never create [M 0x1d1b68:77-96]), the quat-0 placement
      matrix, Collide_PickObj's per-cel walk + admission gate, Break_Kill 0x12b970:59-63.
- [x] the census: pane_pairing_census.rs prints 01/09/0a/0b/14 (0b has ZERO 0x20 rows) —
      120/120 parts on their placements, all game-created, no doubles; the all-SP assertion
      passes.
- [x] the ride wired: world.rs PANE_PART + the ray-set skip (the per-cel stand-in) + the
      node-pass flag; drone.rs rides the kind-2 panes with the death-tick grace behind
      PANE_RIDE; BREAK_WORLD_STOP tied to the switch. Ride-on: dead_pane 2/2, glass_pass 3/3,
      walkreal 09 8/8 green; the 01 real-damage trio dies in a guard-melee cascade (master
      green, bisected) — the switch lands OFF, master's walks stay green.
- [ ] left: settle the 01 capsule/melee cascade, flip PANE_RIDE, un-ignore dead_pane (the
      report's Live checks); the suite run with the ride on.
