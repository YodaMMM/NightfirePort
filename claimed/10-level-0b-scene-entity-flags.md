# 0700000b: which roof-scene objects hide from the scene camera

The roof cutscene creates its own lift and lift_glass objects (Script_CreateEntity -> Control_SetGList). Control_SetGList 0x134960:14-15 hides an object from the scene camera when its glist flags carry 0x40000. Read the disc's glist flags for every object the 0b scenes create (lift 0x02000367, lift_glass 0x02000421, the NIS door, any others) and report which carry 0x40000; if any do and the port still draws them in the scene, name it in the report (draw change is the main agent's). No draw changes on this branch; a census test is welcome.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0b-scene-entity-flags`, report in docs/reports/level-0b-scene-entity-flags.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T06:30Z
