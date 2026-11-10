
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
