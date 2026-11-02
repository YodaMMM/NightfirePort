# consider-explosive-gate-3 — land the game's grenade throw/flee rules with every walk green

Branch from origin/master; merge origin/s/consider-explosive-gate-2 in first.

That branch pins DroneWeap_CanThrowGrenade 0x17a798 to the disassembly and ports the flee gate [M 0x1431c0:40-67]; the 07000011 corridor tests are green with it. It is held because on master it breaks three level walks:
- walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage — fails at tick 2640: "#385 goes on #383's blast (cause 2)" (the fence chain no longer propagates as the walk expects; health 86.1, not a death)
- walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission — Bond runs dry in the pit-squad fight (the branch ignores it)
- walkreal_07000016_live::the_chasers_punch_lands_the_record_damage — the real-damage walk dies (the branch ignores it)

Do:
1. For 15: find why the guards' new throw/flee behaviour changes the #383 → #385 blast chain (a guard's grenade or flight setting off / moving into the fence?) and whether the game would do the same; fix the port or the walk accordingly, with citations.
2. For 16: give the pit-squad walks a player's tactic under the faithful roll (ammo pickups, weapon choice, cover). Real input only, no health/ammo writes.
3. Un-ignore both 16 walks. Run every walkreal_*_live, start_corridor_guard_07000011_live, every grenade test, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level and re-run every walkreal_*_live.
