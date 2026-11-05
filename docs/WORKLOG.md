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

## fx-muzzle-repin
Started 2026-11-05 09:36 UTC. Branch s/fx-muzzle-repin cut from origin/master 811cb3e39.
Task: land s/fx-muzzle-and-trail on current master — merge it, keep gas.rs hidden/born/child/rot
fields on every literal, fix the tests the rebase breaks (RNG stream shift) without removing or
reordering a draw, fix the stale drone.rs comment that says the muzzle draws never fire, run the
full list and the full suite.

## fx-muzzle-repin-2
Started 2026-11-05 11:58 UTC. Branch s/fx-muzzle-repin-2 (continued from origin/s/fx-muzzle-repin, merging origin/master).
Task: finish the muzzle re-pin — the -1 report's "Still red on the merged tree" list is the work:
the walkreal 01/03 walk reds, fix_npc_doors, fix_pickups_trace, start_corridor_guard_07000011, the
unrun walkreal 11/14/16 + npc_doors/fix_guard_misc + the full suite. Remove or reorder no draw;
walks fight back or route around as an opt-in flag in the shared walker; run every binary on the
list and report each result.
Progress 2026-11-05 11:58 UTC: branch cut from origin/s/fx-muzzle-repin (its report read — the
corrected-stream re-derivations are the standing doctrine), pushed. Master merge next.

Progress 2026-11-05 13:28 UTC: master merged clean. walkreal_07000001 GREEN 8/8 whole (110.24 s —
the merge re-shifted the stream past the -1 report's walk deaths). walkreal_07000003 GREEN 6/0/1
whole (312.39 s): the east-lane ladder (lenient per leg + the moat sidestep + the skip, the
audit-voice-triggers-3 law) and the Agent real-damage walk's fight-back via the fights'
head-hunting pick (the torso-nearest taps took #5 12->7 across 12 fights while his volley landed
60.4; one head hit drops him [M 0x145558]); the pre-fight stand and the 6.5 stand-off were
measured and reverted (the stand re-routed the plain walk into an exit wedge; 6.5 changed nothing
— the same t62511 death). fix_pickups_trace_live GREEN 2/0: the corrected stream lands one Raptor
drop INSIDE the slope (y 5.34 under the walking floor at (-11.86, 2.94)) — recorded with its
position (the take ladder gained the drop's-floor stands first); the cheat-refusal symptom's
pins (clip 9, no spare leak, the 4..8 rounds, the cap) all hold. Still red/not run: fix_npc_doors
(the escort settles at (4.56,6.34,6.31), never leads past the shoji doors), start_corridor_guard
d2 (idle Bond survives 1500 ticks), walkreal 11/14/16 + npc_doors/fix_guard_misc unrun, the full
suite not run.

## fx-muzzle-repin-4
Started 2026-11-05 16:52 UTC. Branch s/fx-muzzle-repin-4 cut from origin/s/fx-muzzle-repin-2 (698c79b3f, not master) — origin/master merge next.
Task: redo the muzzle re-pin on the right base (repin-3 was cut from master and holds none of the
muzzle code): after the merge, grep -c shot_flashes crates/nf-game/src/drone.rs and grep -c
init_gt_shot_smoke crates/nf-game/src/gas.rs must both be above 0 (outputs quoted in the report),
then finish fix_npc_doors_live, walkreal_07000011_live and walkreal_07000014_live keeping every
draw; report each binary.
Progress 2026-11-05 16:52 UTC: sandbox reset, repo recloned, branch cut from repin-2, pushed.
