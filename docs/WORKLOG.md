# WORKLOG

## fx-muzzle-and-trail
Started 2026-11-05 00:41 UTC. Branch s/fx-muzzle-and-trails cut from origin/master 17a784980
(already carries the audit-effect-calls merge). Task: port rows 6, 12, 14 of audit-effect-calls's
"Unported calls" table — the turret muzzle smoke (GT_Update 0x180498:356/:530, GunImp_Update
0x1320d8), the bullet trail big puff (Bullet_trail_effect_smoke 0x12cba8:41-53, incl. the
GameState+id gate fix), and the guard muzzle-flash object (DroneWeap_FireWeapon 0x178f68,
kind-0x40 sprite + 3 orientation draws). Keep the game's RNG draws and say what each shifts.
Shell casings out of scope. Pin each with a test through the real path on real level data.
Progress 2026-11-05 03:56 UTC: rows 6/12/14 ported. Row 6: `GasPool::init_gt_shot_smoke`
(GT_Update 0x180498 case-4/case-7 fire tails — the barrel byte +0xf6 ^= 1 alternator, smoke on
the toggled-1 shots, 6 draws, scale 0.05→0.1 life 128 rgba 0x48403080) emitted as
`GTEvent::ShotSmoke` and drained in the Drones GT arm; `GasPool::init_gunimp_smoke` (GunImp_Update
0x1320d8:218-247 overheat smoke, hot && GameState%6==0, barrel-tip base − radius·FRand(1.0) along
the aim) wired before the fire/stop block. Row 12: `GasPool::init_trail_big_puff`
(Bullet_trail_effect_smoke 0x12cba8:29-53 — the +0xf6 != 0 arm: (GameState+(u8)obj[0xec])%15==0,
kind 0xb, scale 0.2→5.0 life 255, 0x404040ff no fade, 6 draws; the +0xf6 latch = the bullet's
FIRST bounce [M DoTrails 0x12d890:397-400], so the gate is bounces>0 — the EE-multiply vel store
fixed by the test) + the trail gates now carry the born phase (Grenade.born = obj+0xec = GameState
at creation [M control_init_object 0x134630:15]). Row 14: DroneWeap_FireWeapon's kind-0x40
muzzle-flash sprite draws [M 0x178f68:189-213 — control_create_object 0x1344e0 is a Mem_Malloc, so
the live game makes the 3 orientation draws (FRand(2π), FRandHalf(0.2), FRandHalf(0.2)) on every
shot] consumed per shot and recorded in `Drone.shot_flashes`; the viewer's AI flash arm builds the
matrix from the REAL jitters (flash_matrix_jitters) instead of the fake seed. Pins: gt.rs
shot_smoke_fires_on_every_other_shot, gas.rs gt_shot_smoke/trail_big_puff six-draw value tests,
nf-view trail_big_puff_live (a dropped 0x36 bounces and switches to the big arm on the %15 gate,
real 07000005 load). ai_live re-seeded 9→16 on the corrected stream (1/11/16/17/19/23 pass every
pin; 16 also passed on the old model). Full suite did not fit the sandbox window; run list in the
report.
