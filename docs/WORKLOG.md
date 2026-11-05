
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
