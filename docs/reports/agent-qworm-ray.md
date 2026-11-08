# agent-qworm-ray: the Q-Worm fires only at a PC under the crosshair

## The game's rule

- `Player_WeaponFiring` 0x1a5460:531-545 [M]: with the trigger held and the Q-Worm (0x58) in
  hand, the cached aim target `BLData+0x880` must be set, of class 0x49 (`PCQWorm_Create`
  0x1b07a8 writes `obj+0xFF = 0x49`), in state 0 (`obj+0xF4`), and the player root within
  `weapon_data+0x1C` (2.0) of the PC's `obj+0x30`. Any failure returns before the pull.
- `+0x880` is written by `Check_Target` 0x1a7810 [M], every 6th frame (`GameState+0x34 % 6`,
  :98), called from `Player_Weapon` 0x1a24c8:278 via `Player_AutoAim`, which runs before
  `Player_WeaponFiring` in `Player_Update` 0x193600:412-413. It casts a 50-unit ray from the head
  along the view, `Collide_RayIntersect(..., flags 0xc)`, and takes the FIRST node of the sorted
  hit list (`Collide_RayIntersect` 0x1ec690:114-160 sorts by fraction); a world hit first leaves
  `+0x880 = 0`.
- The PC's part of that ray [M]: `Collide_PickObj` 0x1eae80 admits the object only with
  `obj+0xF0 & 0x70 == 0` (0x20, plus 0x40 for flags bit 8 and 0x10 for bit 4), a glist with
  collision, and the ray inside its world sphere; `Collide_Intersect` 0x1ebaa8's 0x201 arm then
  tests the glist's own triangles in object space (inverse of `obj+0x90`), one-sided
  (`Intersect_RayGeom` 0x10a8c8:168), and flags bit 8 drops pass-flagged hits.

## The port now

- `UseObjects::aim_pick` (nf-game useobj.rs): the PC mesh ray above. Each PC now carries its
  shape key, its `obj+0x90` (`View_RotTransMatrix`) and its glist flags (`obj+0xF0`,
  `Control_CreateObjEx` 0x1343c8:31-36).
- `Sim::aim_pc` (nf-view scene.rs): written every 6th frame before the firing; the PC is the target
  only when its hit is nearer than the world's (`segment_bullet`, the flags-bit-8 ray) and the
  drones' (the existing Check_Target drone cylinder, now `check_target_drone`).
- `UseObjects::qworm_gate` refuses the pull unless the target is an idle PC within reach.

Disc facts [D]: the two armed PCs (0700000a #688 ch 0x48, 0700000b #297 ch 0x4C) share one
130-triangle glist whose collision spans y ±0.34 about the placement. 07000011 #216 and
07000012 #684 are E-installs (param 2 = 1, param 3 = 0), untouched by the fire gate; 07000012's
glist flags 0x50 also keep it out of the aim ray.

## Tests

- New: `interactables_0700000a_live::the_qworm_fires_only_at_the_pc_mesh_in_reach`: aimed along
  the band no shot; inside the PC's world sphere but off its mesh no shot; on the mesh from 2.18
  out no shot (and the target was the PC); on the mesh from the stand it fires. Red on master
  (the band aim fires) and red with a sphere stand-in pick (no off-mesh aim exists for it).
- Test harness moves: the Q-Worm shots aimed 0.55 above the placement, over the PC's mesh; they
  now aim at +0.2. Two 0a install tests stand at the walked south stand (1.85 from the PC, the
  old teleport stand was 2.18 out). `walk_0700000a_live` used a stance with a partition 0.14 in
  front of the eye; it now uses the south stand. `walk_0700000b_live` holds the aim one 6-frame
  cycle before the pull.
- Green: interactables 0a (14), 0b (14), 11 (16), 12 (12); walkreal 0a (13 + 2 ignored),
  0b (7), 11 (5), 12 (3); walk 0a (3), 0b (4); objectives_0b_start (3); qworm_fire_mode (3);
  qworm_not_on_0c (3); nf-game useobj (7).

## Unproven

- Doors, triggers and other objects in front of the PC are covered only through the world ray's
  dynamic set; the drone arm is the port's 0.4 cylinder, not the drone's own collision.
- The visible PC model was not compared with its collision mesh: if the model draws taller than
  y +0.34, aiming at the drawn top refuses the shot (the game would too, if the data match).
- `Check_Target`'s player-state skip (`obj+0xF6` cases) is not ported; the port refreshes the
  target in every state.
