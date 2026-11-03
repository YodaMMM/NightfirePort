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

## real-health-00agent-08-09-0a-0b (started 2026-11-03 20:52 UTC)

00 Agent (difficulty 3) sisters of the real-damage walks for 07000008, 07000009, 0700000a and
0700000b: same routes and ends on real input, `Drones::difficulty == 3` asserted at each load,
difficulty-3 rows (Plr_DMod, accuracy, Drone_Create's gate) checked against the port [M].
