# WORKLOG

## muzzle-repin-5 (2026-11-06, s/muzzle-repin-5)

Task: re-pin the tuned walks on the muzzle-draw master. Cut from `origin/s/muzzle-on-master`
e5867f04e ("Guards' muzzle flashes, turret smoke and bouncing-round puffs follow the game" on
576909ed6) — the game code is reviewed and rendered, every draw stays. The three flash draws
per guard shot shift the shared RNG stream, so the tuned walks move: fix them test-side only,
by route and fight levers; never loosen a pin tagged [L].

Driver's red list (suite 4387 passed / 19 failed): walkreal 02 real_health_agent, 03 full
route (timeout), 05 door band real_damage, 01 00agent, 06 real_damage, 08 00agent, 0b 00agent,
11 real_damage + _agent, 12 _agent, 15 real_damage + _agent, 16 real_damage, 1b Drake walks;
interactables_07000001 bonus walk; start_corridor_guard_07000011; goggles_draws_live
a_captains_laser_ray (the key reached mode 1); fix_grenade_types stun hold 4361.

Check-first items (stream shift or something else?): 02 "the computer's destruction raises ch
0x48", 15 "mine never exploded under fire", 08 "the shoot-out must kill every end pane", and
the stun hold — report separately.

Plan: build the red binaries, run each whole for a ledger on this master, then re-tune by
route (waypoints/legs) and fight levers (carry, timing, clears) — test-side only.
