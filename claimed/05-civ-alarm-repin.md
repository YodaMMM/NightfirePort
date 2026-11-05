# Re-pin the walks on the civilian alarm fix

Branch from `origin/s/civ-first-attack` and push to `s/civ-alarm-repin`. Do not change game code unless a test exposes a real mismatch with the decompile. Fix the test routes, not the behaviour.

The branch makes scared civilians run `DroneFunc_FirstAttack` and shout. Proven live in PCSX2 on 2026-11-05 with breakpoints on Drone_SM_SetState and DoForcedAttack:
- 07000012, Bond idle at the start: the office worker goes 0x12, 0x85, 0x64 at frame 168, and both corridor guards get msg 0x1f at 228 (one in Alert, one in PlayScript) and fight.
- 07000009, Bond 6 units in front of a lobby security guard: 0x12, 0x3e, 0x89, 0x63, 0x8a, and alarm channel 0x61 goes on around frame 696.

So guards now fight early on the tower levels, and being seen on 09/0A raises the alarm. Tests that assumed otherwise need new routes.

Failing on the branch (full suite 4378 passed, 21 failed before the stealth tests were updated):
- walkreal_0700000a_live: walk_office_floor_qworm_keyboard_hatch and its three real_damage sisters
- walkreal_07000009_live: the_alarm_switch_fails_objective_1, walk_tower_a_start_to_finish and its three real_damage sisters. The route must stay out of the security guards' sight or drop them before they ack, as a player must. "must not raise the alarm" stays asserted.
- walkreal_07000011_live: walk_spawn_to_the_lower_floor_load_point_real_damage and _agent (Bond dies at tick 1011 on the first; he now has to fight from the start)
- walkreal_07000013_live: walk_the_warm_chain_to_the_front_entrance_and_out_real_damage
- walkreal_07000006_live: walk_the_castle_interior_to_the_party_real_damage and _agent
- door_colour_stable_live, goggles_draws_live, scorpion_modes_live: setups that assumed an idle guard at the level start
- start_corridor_guard_07000011_live: confirm it stays green

Real input, no teleports in walks. Run every binary above whole, then the full suite if the window allows. Quote the results in the report.

---
claimed-by: sandbox1 at 2026-11-05T18:33Z
