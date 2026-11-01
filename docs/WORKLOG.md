
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

Session 1 results (2026-11-01): the reach question answered from the data — the port's 4.0 IS the
game's number: 0x4E range +0x1C = 4.0 / 0x4F = 8.0 [L ee_start_05.bin weapon_data 0x2bf150 +
id·0x10C], the death check compares record+0x1C vs travelled [M Bullet_update 0x12e7b0:158], the
speed loads from +0x20 [M Bullet_init 0x12c5a0:187]; the whole life of a watch round is ONE clamped
segment. The contact chain written down: DoTrails' tail walks the charge's obj+0xD0 list (class-5
hitters, 0x4E → −1, 0x4F → −10 off BU+0x102, sparks Emitter_Create(0xc00000a, hit+0x40, hit+0x10))
[M 0x12d890 tail 0x12e2f0-0x12e390]; the contacts come from the collision pass — broad phase the
charge's glist world sphere [M Collide_PickObj 0x1eae80:92-108 + Control_BuildWorldSph 0x133070],
narrow phase the glist mesh [M 0x1ebaa8/0x10a8c8]; the port's 0.5 radius stays [S] with the
fieldprobe check (+0x80/+0x8C) listed. The space-laser impact def fixed: 0xc000010 [M 0x1aa3f8:189;
Emitter_Create 0x17ae00:86 arg order — def = arg 3, pos = arg 4, normal = arg 5]; 0x2d6210 is the
boot-written (0,1,0) constant's ADDRESS [L .data; M 0x1a65d8:190 passes it to Mat_Align2Dir].
Pins: bullet.rs `the_burn_segment_never_passes_the_record_range` (the not-beyond sister) +
the 4a ride's first-contact distance assert (< 4.5 [L]) through the real ride/aim/fire path.
Runs: nf-game 988/0/9; the satchel/laser binaries 42 passed 0 failed (the list in the report).
