# WORKLOG

## fx-muzzle-and-trail
Started 2026-11-05 00:41 UTC. Branch s/fx-muzzle-and-trails cut from origin/master 17a784980
(already carries the audit-effect-calls merge). Task: port rows 6, 12, 14 of audit-effect-calls's
"Unported calls" table — the turret muzzle smoke (GT_Update 0x180498:356/:530, GunImp_Update
0x1320d8), the bullet trail big puff (Bullet_trail_effect_smoke 0x12cba8:41-53, incl. the
GameState+id gate fix), and the guard muzzle-flash object (DroneWeap_FireWeapon 0x178f68,
kind-0x40 sprite + 3 orientation draws). Keep the game's RNG draws and say what each shifts.
Shell casings out of scope. Pin each with a test through the real path on real level data.
