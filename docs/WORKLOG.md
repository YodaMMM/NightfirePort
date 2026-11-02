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

## level-16-pit-squad-faithful-roll (2026-11-02)

Branch s/level-16-pit-squad-faithful-roll from origin/master (e79e7bc9, carries the faithful
grenade roll: DroneWeap_CanThrowGrenade 0x17a798 + the 0x68 flee gate). Goal: the two
07000016 walks the faithful cadence broke — walk_the_chasers_defeat_completes_the_mission
(Bond runs dry 0x11 clip 0 spare 0, ch 0x82 stalls) and the_chasers_punch_lands_the_record_damage
(no-return-fire real-damage walk dies) — get a player's tactic (route pickups, weapon choice,
cover from the throws, real input only, no health/ammo writes) and are un-ignored. Then
walkreal_07000016_live, fix_finale_live, every grenade test, the full suite.
Progress (same session, 2026-11-02 ~22:05): the port's fight laws fixed against the decompile —
ConsiderExplosive's state gate refuses only {0x28,0x2d,0x4a,0x4b,0x53-55,0x63,0x86-88} [M
0x1431c0:40-67, decomp re-verified]; the case-0x10 throw+move gate is the anim class-2 AND
anim-id-0xA pair with DroneWeap_CanThrowGrenade's Rand(2) consumed on every gated tick with
grenades even out of the 8..20 band [M 0x148338 case 0x10 + 0x17a798, decomp re-verified]; the
stun 0x35 is not in Drone_BuildDynamicAwarePoints' registry at all [M 0x138b10 — no 0x35 case].
Master carried none of the three (the gate-2 branch is unmerged); the two 07000016 walks passed
on master only because the unfaithful port skipped the out-of-range rolls and refused the 0x68
flees. With the faithful laws: the plain walk needed one harness fix (the lift-out wait read
`0x82 && disabled` — the game's movie player races a disabled-Bond script through in ~46 frames
[L 748 ticks across frames 4938..4984 for the pack cinema], so a long squad fight's 0x82 edge
lands and the hand-back completes before the first check; the wait now reads the latched 0x82
[D mux #633] and the movie's own hand-back proofs below it) — green 14/14 with it. The punch
walk's fighting approach: the kill order (frag holders first [D #533/#535 p11 0x43]), the
head-box kills (prefer_head, the 6 u window), the airborne-grenade jink, the drop loot (Bond
holds the looted 0x1a 25+43), the vest #596 guard on the loot walk, the per-leg advance-fight
rhythm (the descent walked not fought — the ramp-edge wedge), the leg re-walks after the fights.
State: the whole camp clears; Bond reaches the chaser with 10.4 health and armour 12.4 (a vest
take the loot guard does not cover — the next item, NF_PICKUPLOG points it); the pin needs >30.
grenade_live 4/4 (the flee-gate pins ride along), grenade_rattle 4/4, stun 1/1, ai_live 37/37
(the flee family), nf-game lib 655, fix_finale 4/4, walkreal_07000016 13/14. The full suite is
the next session's first item; the punch walk's vest take + health budget the second.
