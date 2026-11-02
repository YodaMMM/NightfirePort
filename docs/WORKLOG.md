
## sniper-range-accuracy-0c-0d (started 2026-11-02 15:47 UTC)

Branch cut from origin/master 84e64088. Task: weapon 0x1e drones hit too hard from too far — on
0700000d drone #31 (skin 0x5000087 at 160.9, 6.8, -235.1) lands 14-16 per shot from 41.2 u; on 0700000c
drones like #18 (skin 0x5000090) deal 80 total by tick 8700 from ~34 u; the 0c/0d real-health walks
are ignored with these reasons. Plan: (1) pin the game's drone fire law at range [M: DroneWeap_*
accuracy/spread vs distance, fire rate/burst, per-hit damage + falloff, skin accuracy/difficulty
scaling, max engagement range], (2) diff the port at 34-41 u (hit prob, dmg/hit, shots/s) and fix
each delta with citations, (3) un-ignore walkreal_0700000c/0d_live if they finish (cover only, no
health writes), run walkreal_0700000c/0d_live, enemy_accuracy_live, every real-damage walk, full suite.

### sniper-range-accuracy-0c-0d (17:03 UTC)

Pinned the weapon-0x1e law at range [M 0x178f68/0x179740/0x12c5a0/0x139f50/0x1902c8/0x179ba8/0x148338]:
distance-independent hit chance gated by the 120-tick continuous-sight counter, zero cone (0x68
0x400000), no damage falloff (16.0/hit on Easy), 180-tick cadence at >30 u, clip-5 reloads, and the
class-based engage band. Fixed the spawner spawns' stats: they now re-roll the SHARED drone_stats
table (load-time seeded) like DroneSpawner_SpawnDrone's DoModeSettingsNEW [M 0x137cb0] — 0c's real-health
walk survives the drones now (19.8 hp at tick 8700, was dead); its remaining ignore clause is the
yard-climb mantle (movement). 0d's death (tick 6549) survives every law check — the open question is
the port's unbroken LOS for placed drone #31 (the probe grid shows no covered crossing); live checks
listed in docs/reports/sniper-range-accuracy-0c-0d.md. reaction_time's default arm confirmed 10
(FRAME_RATE_DIV = the 1.0 identity), with the evidence note in code.
