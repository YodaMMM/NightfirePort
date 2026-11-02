## level-11-start-fight-real-health (2026-11-02)

Branch s/level-11-start-fight-real-health from origin/master (e79e7bc9).
Goal: un-ignore walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage —
Bond dies at tick 3193 at (10.1, 1.1, -13.8), last hits -3.00 x2 from drone #10 (skin 0x5000008,
weapon 0x1c) at (19.5, 1.0, -11.4), 9.7 u. Plan: read the full damage log; compare start guards'
fire rate/accuracy/damage vs game [M drone weapon code 0x1c; start_corridor_guard_07000011_live
[L] pins]; fix port where it differs, else route the walk with real tactics (no health writes);
run walkreal_07000011_live, start_corridor_guard_07000011_live, all real-damage walks, full suite.
Progress (same session, 2026-11-02 ~21:00): reproduced the real-damage walk on master
(e79e7bc9) — it now dies at tick 3093 on 07000011's west flight B: the return-fire stand
held fire ~143 ticks because the gun line to its target (drone #17) was blocked by drone #18
(1.6 u, weapon 0x19) which landed 15 rounds for 51 health [L the run's NF_RF_TRACE]. Damage
chain verified against the game: weapon 0x19 base 3.5 / 0x1c base 3.0 [L ee_start_05.bin
weapon_data 0x2bf150], Drone_ModBulletDamage returns raw for weapons < 0x62 [M 0x139f50:20-23],
Player_HandlePain x Plr_DMod_{0.5,0.6,1.0} on 0x07000011/12/13/4a [M 0x1902c8:28-46 +
0x1d3a80:164-176] — the port's per-hit damage matches at every distance. Fix shipped: an
opt-in `DamageLog::shoot_blockers` (set by this walk only, under real damage) — when the line
to the fight's target is blocked by a live hostile, the fight retargets the blocker (a player
shoots the guard who walks into his line). Walk then passed 07000011 and died at tick 2789 on
07000012's south corridor: drone #10's weapon-0x1c loop = 8 pellets per trigger loop [L
weapon_data + 0x1c*0x10C + 0x18 = 8; M 0x178f68:233-241] x 2.0 TooClose at 0.6 u [M
0x139f50:24-26] = 45.5 health in one tick, Bond dry (clip 0 spare 0 — the fresh d3 front-end
load's own 7 P2K rounds spent on the lobby guards). Armour-detour tactic tried and reverted
(the detour legs reshuffle the spawn-lobby fights, 93->30 health, died short of the pickup).
Verified: start_corridor_guard_07000011_live 8/8, the topped-up walkreal_07000011 walk green.
The 07000012 corridor tactic is the next session's first item.
