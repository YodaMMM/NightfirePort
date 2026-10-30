# WORKLOG

### level-0c-gate-band-crossing (start, 06:31 UTC, 2026-10-30)

Task: the 0c whole walk crosses the gate band x 55.5..67 on a stand-in: the port's disc
collision is void there [D probe door4 pit map], yet the game's nav path17 links
(49.69, 10.74, -207.29) <-> (63.06, 10.74, -207.54) across it [D]. Find what carries the
player across in the game (a mover, a wire, a collision the port mis-reads) [D the
placements/collision over the band; M the class code], fix the port side, and replace the
stand-in with a real-input leg. The walk must stay green. Run the touched binaries and quote
their totals. Branch from origin/master 0c90eba8.

### level-0c-gate-band-crossing (session 2, 07:31-09:10 UTC, 2026-10-30)

Continued from session 1's probes (the void static, no SS/wire/mover, the grapple
elimination). This session: the sandbox kills background processes between tool calls, so the
fast suite runner cannot stay up here (started twice, died both times; foreground chunks only).
New [D] evidence via two new probes (band8 = the full band-box placement census + the sim
grapple/wire stores; band9 = the placed extents of every turb walkway/staircase/supportbeam +
the pulley): over the band void x 56.08..66.27 there is NO carrier entity — the only placements
in the box are the guard spawn plc60 (62.99, 11.42, -207.43), PlayerStart plc104 on the band
west, and the east oil barrels at band height; the 5 grapple points (all class 0x3a, all named
graple_pulley #308/#346/#500/#508/#511 [D docs/levels/0700000c.md:98]) sit elsewhere (the near
one gp4 (54.27, 15.35, -237.02) is the south yard); the 4 wires are the three silo cables +
the north-yard zipline. The pit floor is obj_turb2_walkway04 plc28 (x 49.29..63.66, y
6.16..7.19) continuing as the sliver obj_turb3_walkway03 plc33 (x 63.66..64.05), then the
chasm x 64.05..66.27 with the resume +3.2 above the pit — the pit route dead-ends. The
staircases are symmetric flanks (turb2 x 49.29..58.73 rising 2.97->11.17; turb4 x 78.14..87.59
rising 0.16->11.20, both z -208.96..-211.67 south of the band) — nothing spans x 59..78. New
[M] reading in the walk's evidence block: path17's band link is not floor evidence — the
game's node-to-node move test (NDrone2_MoveTest 0x155330, the port's nav::move_test mirror)
sweeps the body node-to-node, and a horizontal sweep across a hole hits no wall and passes
trivially; the mid-void guard spawn pins a placement, not a walkable surface. The port's parse
has nothing to fix (its static world equals the disc's declared collision [D collaudit]); the
crossing mechanism stays unpinned by everything the disc declares — the fresh-boot PCSX2 live
check stands. The walk test's stand-in kept (pinned), with two new sister asserts pinning the
pit floor (6.16..7.19 at (59,-207.5)) and the east sliver (6.17..7.19 at (63.9,-207.5)).
walkreal_0700000c_live 2 passed 0 failed (14.48 s); the two new probes 1 passed each.
