
## sniper-range-accuracy-0c-0d (started 2026-11-02 15:47 UTC)

Branch cut from origin/master 84e64088. Task: weapon 0x1e drones hit too hard from too far — on
0700000d drone #31 (skin 0x5000087 at 160.9, 6.8, -235.1) lands 14-16 per shot from 41.2 u; on 0700000c
drones like #18 (skin 0x5000090) deal 80 total by tick 8700 from ~34 u; the 0c/0d real-health walks
are ignored with these reasons. Plan: (1) pin the game's drone fire law at range [M: DroneWeap_*
accuracy/spread vs distance, fire rate/burst, per-hit damage + falloff, skin accuracy/difficulty
scaling, max engagement range], (2) diff the port at 34-41 u (hit prob, dmg/hit, shots/s) and fix
each delta with citations, (3) un-ignore walkreal_0700000c/0d_live if they finish (cover only, no
health writes), run walkreal_0700000c/0d_live, enemy_accuracy_live, every real-damage walk, full suite.
