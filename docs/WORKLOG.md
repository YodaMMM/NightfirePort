## level-4a-every-objective (started 13:31 UTC 2026-10-29)

Task: make crates/nf-view/tests/walkreal_0700004a_live.rs walk the WHOLE mission of level
0700004a from a fresh load on real input (step_bond only; no teleports past blockers, no
channel or state pokes): every objective the mission data lists [D objective records /
Mission_MonitorObjectives 0x1827d8 M], in player order, each asserted complete, ending on
the level exit firing. First list what the current tests already cover and every
stand-in/poke they carry, then add legs for the rest. Every blocker is a port gap: find the
game's behaviour [M/D] and fix it, or list it in the report if drawing only. Full suite,
quote its total line.

Plan: branch pushed from origin/master (1b2309ea) now; downloads (work-slim + ISO) next,
then read the existing walkreal_0700004a_live.rs coverage, the level docs and the mission
data, and build the missing legs.

Progress (14:55 UTC): the mission walk was already green on master (walkreal ride 5.5s,
satchel 7.9s — the level's ONE objective "Gain access to lobby" reveal ch 0x4d asserted,
level exit ch 0x84 → LevelEnd asserted). The real work became GAP-1: the two interact
spawner tests were parked on "the car walls do not block the Drone_PositionVisible stand-in".
Probe (probe_4a_spawn_live) redid the diagnosis: at the parked z −12.4 the rays ARE blocked
and ch 0x17 never fires (the park leaves every plate's volume — the arm never latched); on
the ride line the spawner arms but every template sits 7.6–7.7 away in x/z forever — the
port's min-dist screen was 2-D, and the game's is `Vec_SqDist3D(player+0x30, spot) >=
min*min` [M 0x137cb0:28-31; the 10.0 default 0x1378d0:35-37]. FIXED the metric to 3-D —
the garrison now releases on the ride line (spawn at the arm tick, the far-down templates
pass the sphere at once); `the_floor_spawner_releases_its_drones_on_the_floor_plate`
un-ignored and green + the no-fire health sister moved in (the spawn health is the port's
stats-default 10.0 [M 0x1c7e58; L 07000005] — the old param-5=0x64 [D] reading pinned the
behaviour words, test fixed per the discipline). The fire-across-the-shaft leg RE-PARKED
with the new exact blocker: 0/2400 clear eye→drone rays from the ride line (the shaft core
blocks the x ≈ −4.3 spots [D census]); game-side reachability unverified [S] — live check
in the report. walkreal + satchel re-run green with the garrison live (6.6s / 10.2s).

Progress (17:05 UTC): DONE — the suite chunks ran in the foreground (background dies between
tool calls in this sandbox): all 893 binaries started, aggregate 3433 passed / 7 failed /
197 ignored / 17 binary-timeouts. Failures classified: 3 disc_golden + identity_disc (the
slim-ISO class), fix_0c_pylon_live ×2 re-run GREEN-check FAILED on the pre-fix drone.rs
(master-preexisting), probe_render_pylon_only passes direct with both trees (env). Timeouts:
walkreal_07000003 4p/387s, walkreal_07000008 3p/94.8s, weapon_matrix 6p/133s uncapped (slow,
not broken); the rest the sandbox hang class. Level docs updated (0700004a.md §7 spawner row
+ live checks; audit GAP-1 → RESOLVED). Report at docs/reports/level-4a-every-objective.md.

## level-4a-every-objective-2 — the ride's fire leg across the shaft

Started 18:47 UTC, branch s/level-4a-every-objective-2 from origin/master b25b5f34 (master
lacks docs/WORKLOG.md + docs/reports/ — restored the -1 branch's copies here and appended).
Goal: the parked fire leg `a_spawned_drone_takes_the_riders_fire_across_the_shaft` (0/2400
clear eye→drone rays from the ride line; group-2 spots behind the shaft core x ≈ −4.3 [D
census]). Plan: read the port's bullet/LOS path vs the game's Collide_LineOfSight and the
bullet pick's cel walk [M]; whether the shaft core is portal-crossable in the bullet's
candidate set decides fix-vs-pin.

Progress (20:10-21:40 UTC): the fire leg SOLVED on the real loadout — the parked body's forced
P2K was a DRY gun (4a's fresh kit is the watch 0x10 + 7 rounds + the grapple, no handgun [M
Player_InitWeapon 0x1a7ef8 case 0x700004a → the 0x7000011 kit; the port's load: clip slot 2 = 0,
every carried pool 0 — the state-2 flush reads the empty slot on the switch tick]). The watch is
a real bullet-spawner (damage 10.0, range 99.0, fire_anim_len 0, clip 7 [L ee_start_05.bin
weapon_data + 0x10·0x10C]); the ride's turn/look inputs work (a pegged ride swept the yaw 34.9
rad — the earlier frozen-dyaw reading was the aimed probe's target alternation). The un-parked
`a_spawned_drone_takes_the_riders_fire_across_the_shaft` now rides the line, aims at the
enabled garrison chest whose eye→chest ray is clear under the game's cel-scoped pick [M
Collide_Pick 0x1eb320:125 → Collide_StraddleCels 0x1eac98; Bullet_update 0x12e7b0 disasm
0x12f0cc-0x12f0d8], presses on the converged ticks (semi-auto: one round per press edge [M the
held gate 0x1a5460]) and lands it: 17 presses, clip 7→4, a garrison drone takes the fire
(probe end-to-end: #30 10.0 → −5.0 at t817). GREEN + the no-fire health sister. The bullet
flight now walks the game's cel-scoped candidate set (`segment_bullet_scoped(find_cel(b.pos))`,
bullet.rs — the same walk the PlayerBullet flight next door runs) and the port's dynamic
node_pass=false was checked against the disc: the lift car's glist flags 0x00040000 carry no
0x40 bullet-pass bit [D map41 obj42 "SCRIPTPLAYER_Lift" 838 collision tris; M Collide_Intersect
0x1ebaa8:143-145] — the car is bullet-SOLID in the game too (its shaft side is open; the clear
rays leave through it). The GROUP-2 leg pinned UNREACHABLE by
`the_group2_spots_stay_out_of_the_riders_fire_line` (GREEN): 0/2400 ride ticks with a clear
eye→drone ray under BOTH the flat and the scoped sets (the probe: the scoped first-hit EQUALS
the flat first-hit at every sampled pose; the BFS floods 1-3 cells that include the blocker;
the 110-pose grid across the car footprint 0 flat-clear). Game-side live checks in the report.

Suite evidence (21:0x-21:15 UTC): the FULL suite could not run in this window — the nohup'd
`python3 tools/test/suite.py` died between tool calls (verified: pid gone, empty suite.out) and
the runner's own build phase exceeded the 540s foreground timeout — the sandbox kills background
jobs. The touched-crate evidence, all --test-threads=1 on the final tree (merged with
origin/master 02e306a6 at e17debbf): nf_game 588p/0F, nf_formats 92p/0F, nf_audio 23p/0F,
interact_0700004a_live 8p/0F (the un-parked fire leg + the group-2 pin + the sisters),
stairwell_bullets_07000011_live 2p/0F (the scoped-flight pin), fix_foliage_bullets_live 3p/0F,
searchlight_bullet_07000006_live 2p/0F, portal_eyecel_live 1p/0F, bullet_range_live 3p/0F,
civilian_first_sighting_alert_live 4p/0F (the P2K fire path), walkreal_0700004a_live 1p/0F.
fix_0c_pylon_live ×2 FAILED = the no-GPU wgpu adapter class (NotFound, Backends 0x0) — the -1's
env classification, pre-existing, not this branch.

More touched-path bins green (21:1x UTC): fix_bond_damage_live 6p/0F (the drone-fired rounds —
the Bullet::update scoped change), fix_autoaim_elevator_live 1p/0F, fix_stunner_press_live 3p/0F,
bathhouse_armed_girl_live 2p/0F, fix_grenade_throw_live 4p/0F, camera_beam_satchel_wreck_live
3p/0F. The level docs' fire-leg rows resolved on the branch (0700004a.md + the audit).
