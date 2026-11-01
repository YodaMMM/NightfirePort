
## laser-watch-reach — 2026-11-01

Task: in the port each laser-watch round reaches 4.0 units and the satchel's contact is a 0.5
sphere (a stand-in for the game's mesh check), so lasering a landed satchel from more than about
4.5 units does nothing. Find the game's real reach and contact [M Player_LaserPointer 0x1a65d8,
the laser watch's weapon record (range) and bullet class, Bullet_DoTrails 0x12d890's satchel hit
test (BU+0x102 tail :452-474) — sphere or mesh, what radius], port it with the addresses, and pin:
the burn counts down from the game's maximum distance and not beyond it (sister). Also check the
space laser's spawn_impact(0x2d6210, …) call in drone.rs against Emitter_Create 0x17ae00's
argument order (def, pos, normal) — 0x2d6210 looks like the normal and the effect id should be
0xc000010; fix and pin it. Run the satchel/laser binaries, interactables_0700004a_live,
space_laser_1b_live and nf-game; quote the totals.

Session 1 (2026-11-01): branch from origin/master (6f341d9e); sandbox from the previous task kept
(repo, work/, ISO, target 5.7 GB). WORKLOG started.
