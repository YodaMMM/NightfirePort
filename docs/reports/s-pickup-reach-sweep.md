# s/pickup-reach-sweep

## 07000014 armour #1019

The walk that stopped 1.82 u short of the armour at (-27.29, 0.87, 30.96) was pressed against the
security booth's own counter. The object is SecurityCheckpointA15 (map 54 object 32, placement
flags 0xC047, so CEL geometry the game builds too [M parsemap_block_map_data_static 0x1d1998:92]).
It raises the booth deck to y 0.8 behind a 65-degree face (x -28.72 to -28.44). Window bars at
x -28.37 sit on top of that face (z 30.40..31.96 and 34.43..36.00). The face's top edge is the
"solid 0.64 u east" in the stuck dump [D the object's triangles].

The way onto the deck is the gap with no bars at z 31.96..34.43, where the floor ramps
0.2 → 0.6 → 0.8. The nav web runs node 118 → 121 → 122 through it [D]. The door at
(-27.7, 2.1, 35.8) is a small hatch at head height, not a way in. Walked through the gap, Bond takes
the armour. The resupply's final straight leg cut across the counter instead. That is a fault in
how the walk steers, not a wall the port adds. Pinned by `the_booth_armour_is_taken_through_the_deck_gap`.
Its sister check: the straight line across the counter stops at the face.

## The sweep (crates/nf-view/tests/pickup_reach_sweep_live.rs)

Every placed pickup on the 22 `dev::LEVELS` entries that have a campaign start: 127 pickups, not counting drops or pickups already taken on an earlier walk.
For each one, Bond is placed once, then moved by real input until the game's take test passes
[M Pickup_Update 0x18e148:203-214]. The tries, in order: the straight steer from the nearest nav
nodes, a floor-grid route, a ladder climb, and a start at a ladder's foot. Doors and SimpleScripts
that open with E are used on the way, and Bond crouches under low overhangs. 106 pickups are
reached. The other 21 are listed in `MISS_OK`, each with its cause. None is a wall that only the
port has:

- Behind objects in their load state: the 05 castle exit door mesh (2), the 08 safe (2) and the
  gondola body (1), the 0b helicopter (1), the 0c secret door on ch 0x08 (1), the 12 glass pane
  (1), and the 15 doors locked on ch 0xC8 (3).
- Static CEL geometry with no way in on foot: 0c sealed pocket (1); 14 crate stacks 1.8 high,
  where Bond's jump rises 0.81 (2); 0d ledge 2.7 high (1).
- Routes the sweep does not drive (grapple, climbing down a ladder, a long route with no nav
  node within 25 u): 03 (1), 05 (1), 0c (2), 0d (1), 41 (1).

## Unproven

- The 15 doors on ch 0xC8: no placement or trigger writes 0xC8 [D]. Script writes were not checked.
- The "not walked" entries: whether the game reaches them by grapple, by climbing down a ladder, or
  by a longer route was not tested.
- The jump rise (0.81) was measured on the port. It was not checked against a live capture.

No port code changed, so no walkreal binary was affected.
