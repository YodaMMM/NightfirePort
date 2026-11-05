
## beam-weapon-panels-1b
Started 2026-11-05 16:46 UTC. Branch s/beam-weapon-panels-1b (cut from origin/s/panel-mesh-wip 786875a4f;
origin/master fd8ee385b already an ancestor). Task: the start weapon (slot 51, the Beam) breaks panels
49/50 at tick 0 from the 0700001b walkway spot where the real game holds fire 245 frames and breaks
nothing (meter ~95). s/panel-mesh-wip's triangle stage in held_hit_volumes/sp_bullet_targets is right per
[M Collide_Intersect 0x1ebaa8] but leaves the bug and fails walkreal_0700001b at "console 1 latched".
Read the real fire path (bullet? press vs meter? damage/tick? class bits? obj+0xD0 contact?), compare with
the port's instant 20-damage round, fix the console-vs-panel difference, keep solar_panel_hits_live green,
run walkreal_0700001b_live + solar_panels_live. Lead: 0x33 Phoenix bolt 0x100000/0x200000 arms explode at
hit (0x60007c4 Explode_Create); two panels together = blast through the sphere sweep, check whether the
game's blast reaches panel SPs and where the port's first shot lands at tick 0.

## aim-snap-objects
Started 2026-11-05 19:34 UTC. Branch s/aim-snap-objects (cut from origin/s/panel-aim-snap 08ace8edf). Task:
finish the crosshair aim snap — port the OBJECT side of Player_WeaponInitBullet 0x1a4bf0:154-164's
Collide_RayIntersect aiming-point snap (mask 0x20a / 0x208 with +0x6C & 0x800) [L the PCSX2 breakpoints:
the beam round leaves the muzzle y -1.23, rises to -1.09 by x 3.47, clears the panels; rounds crossing a
panel's 4.77 sphere get no hit entry unless they touch its triangles]. Drones::aim_snap covers level+panels
only today; add drones, doors, wires/fuseboxes, breakables, script entities; read Collide_RayIntersect
0x1ec690 + Collide_Pick; the glass mask bit 2 semantics (aim_snap's glass argument may be inverted); check
+0x6C & 0x800 against the records. 13 red on the branch (suite 4386/13): fusebox_geometry_07000014,
interactables_07000014 (the laser cut), rocket_stop_plane08 x2, rocket_glass08 (live-proven — fix the ray),
rocket_origin (update to the snapped point), walkreal_07000014 x2, walkreal_0700000c real_damage,
walkreal_07000016 x2 (check for the same ray gap first), walkreal_0700001b x2 ("console 1 latched" — new
firing spots above the panels; order 3,5,6... stalls at console 8 from (44.77,-8.66,1.68)). Keep
firing_along_the_walkway_breaks_no_panel green; keep the panel-only triangle stage.

## panel-model-glist

Aim snap: finish on the corrected filter. Branch cut from origin/s/aim-snap-placement fc6e21313
(the aim ray's filter reads the placement word — object flags never carry 0x8000, so the old
filter skipped every static surface and the snap never landed). Green on this tip:
firing_along_the_walkway_breaks_no_panel, the ray stopping at x 6.958338 — the game's own point
[L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62),
object 0]. Red on this tip, each to be fixed by matching the game's ray: rocket_glass08_live /
rocket_stop_plane08_live (the rocket must pass the 08 window plane -96.75), the 07000014 laser
cuts (the wire needs to be in the ray — the game's ray hits object glists [L the 0x1c object at
(139.562, 50.792, -96.706), pos (140.719, 49.5, -96.751), r 1.812]), rocket_origin_live against
the true snapped point. Then the walkreal consoles (0c/14/16/1b) from player spots and strip the
NF_AIMPROBE prints.

Progress (panel-model-glist): the flight's object-pass arm shipped — Bullet_CollisionHandler
0x12ce80's obj+0xF0 & 0x60 rule [M disasm 0x12cf58-0x12cf5c; Control_CreateObjEx 0x1343c8:36
writes obj+0xF0 = glist+0x44 | create param] ported as a per-door bullet_pass (the placement
word's 0x60 bits) applied in flush_world_impacts, with the world impact recorded at the
explosion point. rocket_glass08_live 3/0, rocket_stop_plane08_live 4/0, rocket_origin_live 1/0,
fusebox_geometry_07000014_live 1/0 (all whole, quoted); interactables_07000014_live 9/1 (the
east-wing laser cut needs the aim ray's object stage). Report: docs/reports/panel-model-glist.md.
