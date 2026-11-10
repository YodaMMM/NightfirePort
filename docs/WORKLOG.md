
## level-05-basin-floor — branch cut

Branch cut from origin/master 4c2203cc6. Task: The Exchange's (07000005) valley basin has its floor
east of the lip. Measured on s/level-05-valley-hurt: past the lip at (132.0,14.17,125.37) every point
resolves find_cel = room 14, and room 14's collision build covers the east basin but has NO floor quads
there, so Bond cannot walk down into the valley the game lets him enter; the west basin floor is room 15
and works; the basin is hurt #3's band (x 80.78..168.38, z 96.66..144.68, y ~5.1). s/level-05-valley-hurt
has only a branch cut so far (no report on it yet). Find why the port's room-14 build drops that floor:
compare 07000005.bin's collision data for the room [D] against the port's parse, and the game's FindCel
relink [M build_FindCel 0x1c8cb0] against the port's ScopeGraph. Fix the build with a test that fails on
the old code (a down ray in the basin hits the floor); then un-ignore
walkreal_07000005_live::the_valley_plane_hurts_bond_walked if the walk reaches the plane. Run
walkreal_07000005_live, interactables_07000005_live, and every walkreal_*_live binary (shared collision
build changes).

## level-05-basin-floor — the basin floor was the hurt plane; the 0xE4 fold

The valley basin east of the lip has no room floor on disc: the outdoor room's collision block
(07000005.bin main map block 262, Room_Road_A1 plc 86, 3121 tris) ends at z 125.98, and its
own up-facing floor quads (y 3.5-7) stop at z ~121.5 — the block walk + scatter probes
(`probe_05_collblocks`/`probe_05_floorscatter`) leave the band x 80.78..168.38, z 96.66..144.68
with exactly one floor-level surface: hurt placement 331's own 2-tri plane at y 5.107 (kind 0xE4
"Hurt_B"; the west twin plc 332 at y 4.378). That IS the floor the game walks: `Hurt_Create`
0x17e870 builds a class-0x1C object, `Control_SetGList` :44 attaches the glist (the collision
block) and `build_LinkToRoom` :46 links it to a room, so the player capsule meets the plane as an
object hit and `Player_DealWithObjHit` 0x194458 applies the damage — solid ground and the damage
are one mechanism. The port's `instances(true)` never built kind 0xE4, so the basin had no floor.
Fix: fold the UNGATED (gate param 2 == 0) 0xE4 placements' collision into the instance list, the
0x19/0xd5 precedent (their runs ride the always bucket; the game links each hurt object to the one
room `build_FindCel` 0x1c8cb0 picks — on 05 the east plane resolves to cel 14 with a single
containing candidate). `walkreal_07000005_live::the_basin_floor_exists_past_the_lip` proves red on
master's code (scoped + flat down rays None) and green on the fix (every basin point hits y 5.107,
normal up, through cel 14). The walked valley test is NOT un-ignored: the floor exists but the raw
run+jump drive still holds Bond at the lip (132.0,14.2,125.4) with every direction ray clear — the
lip-edge walk behaviour is a separate bug, ledger untouched. Report:
docs/reports/level-05-basin-floor.md.
