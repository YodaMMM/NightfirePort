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

## level-11-start-fight-real-health-2 (2026-11-02)

Branch s/level-11-start-fight-real-health-2 from origin/s/level-11-start-fight-real-health,
merged origin/master. Goal: the 07000011 real-health walk's last blocker — 07000012's south
corridor (drone #10's 8-pellet loop at 0.86 u, Bond dry). Tried this session, all measured:
the corridor swing to z -13.8 (the pre-batch line) — the straight legs cut the corner to
0.6-2.2 u of the stationary post and the funnel's geometry caps every path at ~2 u [L repro5-8,
all dead at tick 2789]; the BodyArmour #734 detour (opt-in tower2 legs) — died 1.8 u short of
the plate: the detour legs cross the east volleyers' field, a dry return-fire fight there cost
82.5 [L repro3], and with fights suppressed a lobby blast landed -72.0 in one hit with NO
Grenade in flight all walk [L repro9 — the blast rides the bullet/prop path, not the port's
grenade list]; a walk-level grenade jink (opt-in) never fired for the same reason. Reverted all
common/ churn — the branch carries only the prior session's verified shoot_blockers fix and the
merge. The walkreal_07000011 walk stays ignored with the measured reasons; the next levers are
the Deagle #777 (1.74, 0.24, 13.99 [D]) detour and a live check on the real crossing.
Progress (same session, 2026-11-02 ~23:35): the suite ran in chunks under the 10-min tool cap —
walkreal_07000011_live (1 passed, the real-damage walk still ignored with the measured reasons),
start_corridor_guard_07000011_live 8/8, EVERY walkreal_*_live binary run (all ok; 07000014 and
07000015 SIGKILLed = the sandbox's known memory kill, not test failures), nf-game+nf-formats+
nf-audio 136 passed / 3 failed (the 3 allowed disc_golden), the nf-view fix_ set 418 passed /
2 failed (the no-GPU wgpu adapter pair), the nf-view non-live set 330 passed / 1 failed (the
same GPU class), 90 nf-view live binaries (212 tests) green. The remaining ~388 nf-view live
binaries did not fit the window. Tactic findings this session: the corridor swing dies in the
~2 u funnel (the post owns the mouth; no path holds the spread distance); the #734 armour
detour dies to the lobby's east volleyers (a dry fight cost 82.5; a -72.0 blast with no
Grenade in flight — the bullet/prop path, the walk-level jink never fired). Next levers: the
Deagle #777 detour, a live check on the real crossing.
