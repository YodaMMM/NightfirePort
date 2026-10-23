## Your task: `countdown-cutscene-cameras`, the 07000014 (Countdown) cut-scene cameras

User playtest 2026-10-22 (check vent14r): "the cutscenes on this level are fucked. took some f12s". F12s dist/nf-shots/009-07000014.png and 010-07000014.png: the camera shows an empty blue void beside a rock wall, and two dark-blue boxes in front of a rock texture. The camera is in the wrong place, or the scene's geometry and characters are missing.

1. For every script camera in 07000014's opening and in-level scenes: dump the camera keys the port uses (position, target, fov per tick) and the script/stream they come from [D], next to the scene's actors and the map cels they should see.
2. Find what the port gets wrong: the camera key decode, the base matrix the script concats, the scene's own map, or the actors' spawn. Cite the decompile [M Script_Interp 0x1d9a08's camera case / the Camera_* functions].
3. If a PCSX2 read is needed (the camera position for the first frames of a scene), say exactly which address to read. The main agent renders and compares.
