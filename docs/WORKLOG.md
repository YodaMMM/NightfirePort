## real-health-00agent-05-07 (started 2026-11-03 13:25 UTC)

Branch cut from origin/master 01917076. origin/s/real-health-agent-difficulty-01-07 is already an
ancestor of master (merge no-op), but master's 2879088f REVERTED its seven Agent sisters — the
05/07 Agent sisters are restored here from f1b37c21 (the two test files differ from that commit
only by the revert). Task: add the 00 Agent sisters (`..._00agent`, difficulty 3) of the 05/07
real-damage walks — the same route and end, `Drones::difficulty == 3` asserted at load, the
front-end's launch path [M C_SBNFDFCTY 0x216ca8:15]. Check the d3 rows: the pain scale
(Player_HandlePain 0x1902c8, Plr_DMod_*), the accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88),
the min-difficulty spawns (Drone_Create 0x136be0:49), any 00-only gates. Fix the port where it
differs; otherwise improve the walks' tactics. Real input only; shared helpers only behind a
per-level opt-in. Run walkreal_07000005/7_live and nf-game.

## real-health-00agent-05-07 (progress 2026-11-03 14:45 UTC)

Both 00 Agent sisters GREEN: 05 `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent`
(56.0 health — the two 0x24 sniper stands at the spawn, the gallery-sentry shoot_point stands from
the corridor mouth, back_off 4.0, the 48/18 shooters_window, the head-hunt wrapper) and 07
`walk_the_party_start_to_finish_real_damage_00agent` (passed as-loaded — the route's own cover
holds at d3). The Agent (d2) sisters byte-exact restores of the difficulty-branch runs (05: 57.6
[L the difficulty report]; 07 green). walkreal_07000005_live 14/14, walkreal_07000007_live 5/5,
nf-game+nf-formats+nf-audio 136 passed 3 failed (the allowed slim-ISO disc_golden), the FULL
walkreal_*_live sweep 25 binaries 0 failed. Shared helpers: the opt-in DamageLog::shooters_window
(48/18, read in shooters()) and the additive Walker::reload_full/shoot_bounded (no existing caller
changes) — the sweep is the rule-7 proof.

## real-health-00agent-08-09-0a-0b (done 2026-11-03 21:58 UTC)

All four 00 Agent (d3) sisters GREEN on real input: 09 `walk_tower_a_start_to_finish_real_damage_00agent`
(100.0, 0 hits) and 0a `walk_office_floor_qworm_keyboard_hatch_real_damage_00agent` (100.0, 0 hits)
pass as-loaded — the 09-0b group's d3 accuracy row is the SAME 1.0 Agent runs [M ReadTuningVars
0x1d3a80 cases 0x7000009-a-b]; 0b `walk_fresh_spawn_to_the_mission_end_real_damage_00agent` (65.5)
passes on the route's own fight package. 08 `walk_the_whole_mission_on_real_input_real_damage_00agent`
(end 31.5) needed the player's d3 tactics, gated `difficulty >= 3` in the level's own test file: the
mid-west doorway stand (kills #15 for 0 off the open long link), the link walk-out stand, the
platform-mouth stand (the pair died in the boarding legs before the old stand ran), back_off 4.0 and
reload_full. The d3 Hind has hp 598 = DiffMod 2.0 [M Copter_Create 0x17d340:76-90] and takes SIX
rockets (the part walk adds raw hitdata damage, no difficulty multiplier [M Copter_Update
0x17dbb8:277-320]) — all six landed (606 >= 598). The d1/d2 sisters byte-exact (their timelines
unchanged). Sweep: 25 walkreal binaries 0 failed; nf-game+nf-formats+nf-audio 136 passed 3 failed
(the allowed slim-ISO disc_golden).
