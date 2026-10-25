

## level-11-stairwell-bullet-wall

Branch `s/level-11-stairwell-bullet-wall` (from origin/master). Window 08:46-10:45 UTC.

Task: 07000011 stairwell — rounds at the abseiling guards stop on something invisible in the
well; decals land on the far wall. Three Bond spots given; rope guards at (18.895, 46.516,
1.434) / (19.745, 46.580, 0.363) [D]. (1) Cast the bullet path from each spot toward the rope
guard, list every triangle/object the port's collision meets (owner, placement kind, glist
flags, material/surface byte, collision class), find which one the game's bullet passes
[M Bullet_update 0x12e7b0 / Bullet_CollisionHandler 0x12ce80, the pick mask, Collide_Pick
0x1eb320 node/surface drops, pass flags surface 0x40/0x80, 0x10 render-only, glass 0x10].
(2) Fix [M] + real-input test from one spot (round reaches and damages the guard); sister:
a round into a real wall still stops.

Progress:
- (08:48) Sandbox survived (repo + work/ + ISO + rust present); branch created and pushed.
