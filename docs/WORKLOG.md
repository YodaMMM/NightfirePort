# WORKLOG

## gas-rng-draws-2
Started 2026-11-03 05:10 UTC. Branch s/gas-rng-draws-2 cut from origin/master 6f16146e, then
origin/s/gas-rng-draws merged in (the task's base). Task: land the gas RNG draws without
breaking the level runs — prove which RNG the game's gas init draws from [M the gas init
decompile]: the shared drone stream or a separate one; if separate, give the port's gas pool
its own Rng (most of the 20 reds should clear); if shared, re-check every red (a walk that no
longer completes is a real regression; only legitimately-moved digests/counts may re-pin,
citing the decompile). Run every listed red file + goggles_render_live; all green; then the
full suite.

Work log:
- (05:10) branch start, WORKLOG entry; merging the base branch next.
- (05:12) the re-land: reverted the two reverts (2856f627 + 0ebfee9a) — the gas draws and the
  goggles fix are back; auto-merge clean.
- (05:15) the RNG provenance PROVEN: the game's gas init draws from THE SHARED STREAM —
  Rand_FRand 0x1e3780 / Rand_Rand 0x1e3738 advance the one global SEED_X/SEED_Y pair, and the
  same pair serves DroneWeap_DoBulletAccuracy 0x179740, Bullet_init 0x12c5a0 and every AI roll;
  the port's drone::Rng implements exactly that pair (18000/30903). Task option 2 applies:
  re-check the reds, no blind re-pins.
- (05:20) the red sweep: 17 failures across the 16 listed binaries + goggles (goggles 8/8
  green). Triage so far:
  - mp_perf_16 digest: re-pinned 0x89cbfcb6ae45c2ac -> 0xfce3e6fa7c122c43 with the [M] cite
    (the bots' Gas_Init sites draw from the shared stream now).
  - hold_npcs: the LAW is intact (held exactly 0.0000 at 240/600/1200/2400, re-measured); the
    unheld window 240->600 (240 landed inside a standing pause of the shifted wander phase —
    0.003 u at 240, 0.63 u by 600). GREEN.
  - sight_frustum x2: placement 261's record is present=false from the intro on — he carries
    prop 0x59 and the GO_TO_GOAL arrival tail deletes him [M 0x165640:47-82]; the new stream
    makes him finish his route during the intro. The game's own law; the test's census must be
    re-derived (the test's own note says so) — left for the next message.
  - walkreal_01/03 real-damage deaths: FRIENDLY NPCs (skin 0x5000036) kill Bond point-blank
    (d 0.6-1.0) — forced hits under 3 u are the game's law [M DroneWeap_DoBulletAccuracy
    0x179740:113-115]; the fixed walk routes get unlucky on the correct stream. Routes need
    re-tuning — left for the next message.
- (06:05) nf-game lib green; ai_live 37/0 green (the base branch's re-pins hold).

## gas-rng-draws-3
Started 2026-11-04 13:16 UTC. Branch s/gas-rng-draws-3 cut from origin/s/gas-rng-draws-2
3083a2ca, then origin/master merged in. Task: finish landing the gas RNG draws — turn the
remaining reds green (sight_frustum_live 2, walkreal_01/02/03/14/16/1b, fix_blast_scripts_live,
fix_finale_live, hostage_killer_07000002_live, enemy_accuracy_live), re-routing walks the way a
player would adapt (never forcing state), re-pinning only values the stream legitimately moves.
- (13:18) branch created, master merged clean, initial checkpoint pushed.
- (14:35) sight_frustum_live 2/2 GREEN — census re-derived onto staying guards 158/161; 261 is
  deleted before tick 120 on the shared stream (prop 0x59 GO_TO_GOAL arrival tail [M 0x165640:47-82]).
- (14:35) hostage_killer_07000002_live 5/5 GREEN — the ch-0x19 opener pin now accepts the captor's
  silent-first 0x2E roll (no voice; step-1 advance sends to the hostage; her expected reply opens)
  [M 0x14a8b8 Rand(6) + D 0x2E.sfx=0 + M 0x141b00 + M 0x144340:10-24] — the same exception the
  repeat rule already carried, on the first turn.
- (14:35) fix_blast_scripts type-0 leg diagnosed (fix next message): the teleport spot
  (1.9717, -0.4, -2.256) is a VOID (no floor within 35u on any ray; Bond falls forever, grounded
  false); destroy[10] (#470) is a zero-extent box (lo==hi==origin, the boundless bigdoor01 leaf);
  the P2K record has spread 0.0 [L ee_snapshot weapon_data +0x24] so the shot dir is exactly
  `look` — and the test's aim_at treats pitch as radians while the game's pitch is half-turns
  [M player.rs look(): p = pitch*HALF_PI], so the shot dives -34.3 deg instead of the intended
  -21.8, passing 1.45u from the origin — outside the degenerate-box 0.1 tolerance [M
  breakable.rs seg_box_t lo==hi arm]. Needs a re-derived reachable spot (a real floor) or a
  compensated aim; geometry, not stream.

## gas-rng-draws-4 (started 2026-11-04 16:49 UTC)

Branch s/gas-rng-draws-4 cut from origin/s/gas-rng-draws-3 (b6dd3a47e), merged origin/master
(b21edd082) in. Task: finish the gas-init draws on the shared RNG stream [M 0x1aacc8:72-80] —
the report-3 list in order: enemy_accuracy's per-setup re-pin first (8 min), then the walk
re-routes (01 contact route + friendly-death stretch, 02 to_the_exit, 03 east-lane wedge +
friendly death, 14 the corridor-door chain at (30.3,35.4), 16 punch pin + defeat walk, 1b
Drake's defeat), fix_blast_scripts' type-0 leg (geometry-dead: the teleport spot is a void,
destroy[10] is a zero-extent box), fix_finale's 16 shelter pulse. Real input only, no
assertion weakened without a decompile address, helpers opt-in per level. Suite via
tools/test/suite.py (background from the start) + chunk_suite.py for the final tree. Sandbox
expires ~18:46 UTC; stop-new-work 18:26 UTC.
Round 2 (2026-11-04 17:14-18:40 UTC). enemy_accuracy re-pinned green (two 0x12 cells: d1 d5
8 was 15, d2 d5 25 was 17 [M 0x1aacc8]; binary 5/0+4i). fix_blast_scripts: aim_at pitch now
half-turns [M player.rs look()], the type-0 leg ignored with the measured geometry ledger
(point box at the origin, no floor within 56 u, P2K range 50 [L 0x2bf368]); binary 3/0+1i. 01:
the doorway lure play opened to d1 (AND fires 1974), lip/release/pre-clear gates opened, a
0x500003a far-field pre-clear added; three later reds carry fresh ledgers (7712 d1 west
corridor, 4162 d3 far pair, 8502 d2 stairwell). Not done: 02/03/14/16/1b re-routes (16 12/2+2i,
1b 15/2+2i, 03 4/2+1i fresh), the finale pulse leg (fresh ledger), the full walkreal sweep, the
chunk-suite total.

Branch s/gas-rng-draws-5 cut from origin/s/gas-rng-draws-4 (7a99e04a9), merged origin/master
in. Task: finish the gas RNG draw landing — the report-4 list in order: the walk re-routes
(01's three remaining reds with fresh ledgers, 02 to_the_exit, 03 the friendly death + east-lane
wedge, 14 the corridor-door chain at (30.3,35.4), 16 the punch pin + defeat walk, 1b Drake's
defeat), fix_finale's 16 shelter pulse (Bond died tick 6493 at (-14.50,-22.00,-170.18) before
pulse 2's window), then every walkreal_*_live whole and the full suite via
tools/test/chunk_suite.py. Sandbox expires ~05:16 UTC; stop-new-work 04:56 UTC.

## gas-rng-draws-6
Started 2026-11-05 13:24 UTC. Branch s/gas-rng-draws-6 cut from origin/s/gas-rng-draws-5
(e4e50af06), merging origin/master next. Task: finish the gas RNG landing — read report-5, the
03 and 14 walks still carry ledgers; land what is left and finish with a full suite pass
(quoting the total line). Do not weaken a decompile-backed test to pass a walk. Sandbox
expires ~15:22 UTC; stop-new-work 15:02 UTC.

Work log:
- (13:24) branch start, WORKLOG entry; reading report-5 for the named next steps, then the
  master merge.
- (13:30) the master merge: gas draws carry through (init_ex/init_ex_rot take rng); master's
  newly-merged Gas_Init sites re-based to the real draws (the laser row-11 puff 0x1cf1ec, the
  buoyancy row-7 puff 0x1d6eb4, the flicker spark 0x1abc3c, the water-ripple callers 0x17c118);
  the kinds-9/10 hidden emitters land with the counter-hash child stand-in (no game caller in
  scope); mp_perf_16's pin carries both parents' reasons.
- (13:57) 03 d2 real-damage GREEN: the probe (#[ignore]d, kept) dumped the door-8 cluster's
  LOAD spots (#18 (27.92,6.25,-5.57), #2 (30.69,6.25,-7.55), #19 (37.01,6.25,-15.04), all
  disabled 0x1; door #8 (35.62,6.51,2.71) closed) — the mouth-clear re-anchored to the load
  band + a bounded hinge-side settle loop. Whole binary 6/0+2i (was 5/1+1i).
- (14:29) 14 GREEN 9/0+1i (was 4/5+1i): the west nudge past the door-wall jamb (both chain
  copies), the descent landing pin sampled before clear_hostiles, the delta exit E pulses +
  the pocket hostile clear (the d1 use ray was eaten by live guards).
- (14:38-49) the sweep: mp_perf_16 re-pinned 0x986f_90e8_e5d6_b12f kills 35 (2/0),
  walkreal_02 3/0+1i, walkreal_16 14/0+2i, goggles 8/0, walkreal_01 5/3 (the draws-5 state,
  no regression), walkreal_1b 15/2+2i (Drake hp 60 after 9000 fight ticks from the hold spot —
  the shots don't land on the merged stream; the report names the re-derivation).
- (14:52) the report; the full suite did not fit (chunk_suite ~3 h vs the ~2 h window).

## gas-rng-draws-7
Started 2026-11-06 02:56 UTC. Branch s/gas-rng-draws-7 (cut from origin/s/gas-rng-draws-6 6bd978d11). Task: finish
landing the gas draws — merge master first (the voice draws landed since), keep every draw, fix 1b's Drake fight
(both walk_arrival_to_drakes_defeat and its real-health sister: Drake hp 60.0 after 9000 fight ticks from the hold
spot [L draws-6]), then run every walkreal binary and report each.
Progress 2026-11-06 03:20 UTC: master merged (402e4f57c, one test-file conflict resolved to
the gas branch's generalized doorway blocks; every gas draw kept), the FULL walkreal list run
whole on the merged tree (26 binaries quoted in the report — 03/14 stay green, the task's
named gap closed), 1b's Drake fight root-caused with probe data: the merged stream's Drake
hovers y 4.2-7.6 above the deck while every reachable hold sits below it and the fight's
level-look fire gate cannot hit up — a static re-anchored hold cannot fix this stream; the
next lever is the rise-window intercept (t0-360, ray true at t0, blocked by the last_tap=0
init) or a floating pursuit. 1b (2), 016 (2), 01 d3 (1) remain red — all carried, ledgers in
the report.

## level-12-engage-range
Started 2026-11-05 11:08 UTC. Branch s/level-12-engage-range (cut from origin/master f68a72216).
Task: s/level-12-invisible-wall-spots found that after the real alarm on 07000012, drone #6
holds at the corridor mouth with clear sight at 9-10 u and #7 stays in cover, so nobody
reaches the cafeteria. Compare `engage_range` for skins 0x5000008 and 0x5000006 with the
game's Drone+0x1A0 [M] and the combat_state advance rule. If the port holds where the game
advances, fix it. Then test the no-sight advance against the corner at (22.47,36.64): does it
stop short of the game's arrival radius? Run the 12 walks and report them.

Progress 2026-11-05 11:08 UTC: branch cut, WORKLOG started. Sandbox survived (repo+work+ISO
present, target/ 5.2G warm from the previous session; closed out level-1b-platform-touch's
ntfy-failed report section on that branch first).

Progress 2026-11-05 11:10-12:15 UTC: the comparison, [M-cited]:
- engage_range VALUES match. The game's sight-class switch `NDrone2_DefaultInit` 0x14b300
  (switch(piVar19[0x1b]), the LAB_0014b8a4 region) case 2: +0xF0 = FRand(3.0)+12.0 = 12-15,
  +0xE8 = 16.0, +0xFC = +0xF0+1.0+FRand(3.0); cases 1/3/4/5 and the default match the port's
  table (drone.rs:1985-1988, 2017) value-for-value; the only per-level overrides are for
  0x7000011/0x7000014 — none for 0x7000012. Both skins (plc #5 = 0x5000006, #6 = 0x5000008)
  carry param 14 = 0x2 → the same 12-15 band.
- The advance RULE matches. `DroneFunc_CombatState` 0x148338: COMBAT (case 0) returns 0x5a iff
  +0xF0 <= +0x1A0; COMBAT_OUT_OF_RANGE (case 2) returns 0x58 when +0x1A0 < +0xF0; the fire
  states (cases 0x10/0x18) advance iff routed (MoveToObject result < 4 or == 10) && +0xF0 <=
  +0x1A0; the MoveToObject radius arg is min(engage, 6.0) (0x40c00000). The port
  (drone.rs:10723-10742, 10759, 10787) implements the same arms; +0x1A0 = the 3D distance
  opponent-obj+0x30 to own-obj+0x30 [M Drone_GetOpponentInfo 0x138f68] = the port's self.dist
  (drone.rs:11361). So the port does NOT hold where the game advances: at dist 9-10 with
  engage 12-15 BOTH hold in the fire states, and both advance only when dist >= engage.
- The FOUND difference (goal-move arrival radius): the game's MoveToGoalPosition arrives when
  the route distance drops under route+0x8C = Drone+0x8EC [M 0x1517d8:49-52], which
  `SetupRouteToPosition` writes from the goal's radius param on every position route build
  [M 0x158b10 `*(float*)(puVar9+0x46) = param_1`; 0 → 2.0], `DefaultInit` seeds 2.0
  [M 0x14b300:579], and `SetupRouteToObject` never rewrites [M 0x158948 — no route+0x8C store].
  Per-caller radii: the alarm-point goal = the AIPoint's own +0x14 accuracy [M 0x153008:20-23,
  45-48], FindRunToPoint the same [M 0x153168:39-43], sound/drone alerts 2.0 [M 0x154aa8/0x154b20],
  FindCover 0.5/1.0 by corner class [M 0x152548:174-181], ambush cover 0.5 [M 0x152958:207-208],
  safety 1.0 [M 0x152250:126-127], mission fresh 1.0 / retry 2.0 [M 0x153a30:141-142, 168-169],
  the closer-position probe 3.0 [M 0x155e10:30], interrogate assist 10.0 [M 0x15add0]. The
  port arrives at a hardcoded 2.0 everywhere (drone.rs:5852) — the AIPoint accuracy field is
  even parsed (aipoint.rs `accuracy`) but unused by the goal paths.
- Fix attempted and REVERTED. Two forms (goal-path radii only; + the persistent route+0x8EC
  carry into MoveToObject) — BOTH turned walkreal_07000001_live's two real-damage walks red
  (Bond dies; walker.rs:279 [real-damage] panic): the true radii bring guards closer to their
  goals (mission 1.0, alarm accuracies < 2.0) than the walks' damage balance, which was tuned
  on the 2.0 stand-in. Baseline proof: stashed patch → the 001 test passes (4.6 s), unstashed
  → fails. Left out of the tree per the merge rules; the exact patch is described here and in
  the report for the next session, together with the cross-level walk rebalance it needs.
- The corner question (22.47,36.64): the game's no-sight arrival radius is the GOAL's radius —
  for the 07000012 alarm response the sound/drone-alert goals carry 2.0 [M 0x154aa8/0x154b20],
  so the port's 2.0 does NOT stop short of the game's arrival radius for those; only goals
  whose radius ≠ 2.0 (AI-point accuracies, mission 1.0, interrogate 10.0) diverge today. A
  live check (below) pins the real game's stop distance at the corner.
- Walks + suites on the final (reverted = master-behavior) tree: the 12 level walks
  walkreal_0700000{01,02,03,04,05,06,07,08,09,0a,0b,0c}_live = 89 passed / 0 failed / 5
  ignored; walkreal_07000012_live 3/0, corridor_guards_07000012_live 1/0,
  enemies_07000012_live 3/0; nf-game full suite 1077 passed / 0 failed / 8 ignored;
  nf-formats+nf-audio green except the 3 expected disc_golden slim-ISO failures.

## guard-goal-arrival-radii
Started 2026-11-05 18:12 UTC. Branch s/guard-goal-arrival-radii (master 2f0c88e2c + origin/s/level-12-engage-range
merged). Task: guards arrive under each goal's own radius — route+0x8C = Drone+0x8EC, seeded 2.0 [M 0x14b300:579],
rewritten by SetupRouteToPosition from the goal's radius (0 -> 2.0) [M 0x158b10], NOT touched by SetupRouteToObject
[M 0x158948]; MoveToGoalPosition arrives under it [M 0x1517d8:49-52]. Port uses flat 2.0 (drone.rs move_to_goal +
move_to_object). Census [M]: alarm point + run-to point = AiPoint +0x14 accuracy; sound/drone alert 2.0; cover 0.5/1.0;
ambush 0.5; safety 1.0; mission fresh 1.0 / retry 2.0; closer-probe 3.0; interrogate 10.0. level-12's fix regressed the
001 real-damage walks (Bond dies) and was reverted — form 1 first, walks re-routed per-level opt-in, never weaken.

## guard-radii-finish
Started 2026-11-06 01:49 UTC. Branch s/guard-radii-finish (origin/s/guard-goal-arrival-radii 70c2f9e57 merged with
origin/master c4c00c7ec, clean merge). Task: finish guard-goal-arrival-radii — the radii are the game's, fix the red
real-damage walks test-side per walk, decide the ai_live cover pin from its [L] source (guard_takes_cover_like_the_game
was measured live; if it disagrees with radius 0.5, find out why before changing it), run each binary whole and quote
results.

## guard-radii-01-d3
Started 2026-11-06 03:30 UTC. Branch s/guard-radii-01-d3 (origin/s/guard-radii-finish e88867f93, already contains
origin/master — merge was a no-op). Task: walkreal_07000001_live ..._real_damage_00agent — the one red in that binary
(guard 11's threshold fight runs the pistol dry, the fall-through shoot flanks the door dry, lip guard #5 0x5000038
24 hp takes the last 24). Levers from the finish report item 5: resupply BEFORE the hall phase at difficulty 3, and
re-run the lip block's scan once the escort crossing begins. Test-side only, helpers opt-in, radii/damage/health
untouched.
  Results (2026-11-06 04:47 UTC): both levers shipped d3-gated in
  crates/nf-view/tests/walkreal_07000001_live.rs — (1) the pre-hall block now sweeps the
  corridor's resupply boxes onto the wheel at d3 (repeated resupply() calls; the taken box is
  gone so each call advances); (2) the door-lip scan/fight extracted verbatim into
  door_lip_scan/door_lip_fight and re-run as the escort crossing begins — the post-AND poll
  (≤20 × 30-tick waits from the hold line) catches #5 the tick he surfaces (iter traces: the
  first lip scan missed him still-disabled, the pre-clear leg then ate his 0.9-u volley). A
  second escort-loop re-scan sits before the follow rounds as the late net. walkreal_07000001_live
  whole 8 passed / 0 failed (40.19 s); the d3 walk's death 70165 → level_exit 07000002 at tick
  10042 with 69.4 health (15 hits for 30.6, 10 return-fire fights; AND 3679, lip re-scan +2
  polls found [5]). d1/d2 byte-paths untouched (their walks stay green in the same binary).
  nf_game lib 683/0/5. Full suite chunked (background jobs die in this sandbox): 357/1134
  binaries, 1174 passed / 4 failed / 113 ignored — the 4: missing_scenery_live ×2 +
  glass_guards_14_live ×1 (wgpu `adapter: NotFound` — this recycled sandbox has no GPU
  adapter), enemy_accuracy_live ×1 (rc 124 — my chunked 240 s per-binary limit vs the suite's
  documented 600 s; net_soak_faults_live likewise starved, direct run exits ok).

## guard-batch-repin

Branch cut from origin/s/guard-batch 1c0249b79 (per-goal arrival radii + guard rounds through
glass, suite there 4399 passed / 8 failed). Task: first the two non-walk reds —
enemy_accuracy_live astronauts_hit_like_the_chain ("diff 2 d 5: 1 shots in 10 s, interval 180":
did the astronaut's +0xc4 mode 0x1d change alter his firing?) and kiko_button_real_walk_live
("the pit squad member #14 dies to Bond's fire") — fix with a citation if game-code faults.
Then the walks test-side: walkreal 16 real_damage, 01 00agent, 03 real_damage, 13 real_damage,
08 00agent (the shoot-out must kill every end pane), 06 real_damage; no [L] pin loosened; each
binary run whole with quoted results.
Progress: both non-walk reds reproduced. The astronaut red is NOT the +0xc4 change (verified
bisect: reverting both attack_mode lines changes nothing; so does reverting the fire-path
glass_pass — which DOES flip the test green, so the branch's fire-path glass pass causes it).
The glass law itself matches the decomp [M Collide_Update 0x1ea398:222-229 — the +0x9a bit-0x20
query forces the pass on surface-0x10 contacts; raw-byte equality, the port's & 0x3f variant
tried and not the fix]. Mechanism: the glass-passed 0700001b plasma rounds live their 0x3b-family
fuse (14400 ticks) through the harness's 4000-tick settle and the measured window; their DoTrails
flyby draws shift the shared RNG stream the sight acquisition rolls ride — k=0 never acquires in
2400 ticks at diff 2 d 5 (los false throughout), k=1 acquires late and fires once. Bond is healthy
at the measure start (health 100, substate 8). The kiko red (the pit squad member's shoot budget
expires) reproduced, not diagnosed. The six walks untouched. All further work next session.
## guard-radii-16-chasers
Started 2026-11-06 03:42 UTC. Branch s/guard-radii-16-chasers (origin/s/guard-radii-finish e88867f93, master already
merged). Task: the 016 chasers reds under the game's guard radii — the_chasers_punch_lands_the_record_damage and
walk_the_chasers_defeat_completes_the_mission_on_real_damage; kill the chip shooters #6/#7/#9 in the early phases while
exposed at 4.6-13 u, route the defeat walk's end around the hurt volume at (-5.8, -163) (identified from placements [D]
first); radii are the game's, test-side only, helpers opt-in; keep the d2/d3 sisters' measured ignores unless they go
green; run walkreal_07000016_live whole and quote it, then nf-game.

## gas-draws-8
Started 2026-11-06 07:55 UTC. Branch s/gas-draws-8 cut from origin/s/gas-rng-draws-7 (562de3531
merge state), origin/master and origin/s/guard-lines-finish merged in. Task: the 1b Drake pair —
verify the merged-stream hover band (x 9-17.8, y 4.2-7.6, states 0xc0/0xc1) against the
decompile, then either the rise-window fight (t0-360) or a climbing pursuit; 16 chasers pair and
01 d3 come from guard-lines-finish, not redone.
Outcome (same session, 08:55 UTC): the Drake hover verified against the decompile — the game's
own astronaut-combat strafe [M 0x154618:80-105 / 0x174550:15-23], the port matches line for
line, no game fix; the fight fixed with the opt-in pitched zero-g mode (aim up, no height
chase, 15 u cover ladder) — 1b whole 17/0/2i, the pair green. guard-lines-finish merged (three
conflicts by hand, both sides kept); the 16 punch rides the defeat walk's full economy — 16
whole 14/0/2i. The 01 d3 red moved to tick 5974 (the ch-0xF slab crossing, arrival health ~10,
ledger in the report). The master wave (walk un-ignores + drone.rs cover-lost) × the gas
stream reds 14 tests on 11 levels — all run whole and tabled in the report; grenade_live's
0x68 flee red reproduces on pristine gas-7 (inherited).

## gas-guard-repin (2026-11-06)

Branch from s/gas-draws-8 (7e8051e18), merging current origin/master. Carries the guard radii,
guard glass and gas work on one branch. Re-measuring the §5 red list first (master moved: jump
landings force two steps, landing/step probes read water, the sighted guard ignores sound).
Merged origin/master (d6245281a; scene.rs ripple comment/damage_log union/mp_perf digest
conflicts by hand, the gas draws kept). grenade_live's 0x68 flee: the port's take-once
bounce queue deviated from the game's per-frame aware-point walk [M 0x1435f0:20-52 +
0x138b10:18-19] — replaced with the live-object per-frame scan; the test's pin re-pinned
to the game's own sequence (0x2d flight refused, 0x68 from the bounce). The gas-stream
frag rolls AWAY (probe: d² 160→177 > 144) so the delivery premise stays red — the
corridor rebound is the lever. Re-measured all 26 walkreal binaries + the door/guard/AI
sets: 202/23/15 total; 1b holds 17/0, 02's lob holds green, 04 (7/3 stats trio) and 16
(11/3 chasers) and 11 (2/3) regressed with the master wave.
## guard-repin-master
Started 2026-11-06 10:05 UTC. Branch s/guard-repin-master (from origin/s/guard-on-master
6f8ce18bb, the guard radii + glass work on current master, full suite 4417/5). Task: triage
the five reds (03 real_damage_agent death 51805; 13 warm-chain "the lobby west mid" 6000-cap
+ a 2627 death at (208.62,1.04,-229.64); 0d c5 door "the blind strip east 4 must walk"; 06
castle interior death 5217 at (20.27,41.13,-37.97); 08 whole-mission 00agent "the shoot-out
must kill every end pane") — for each, walk-tuned-to-old-positions vs guard-behaves-wrongly.
Route/timing levers only; no [L] loosening, no ignores, no game-code changes to suit a walk.
Finish battery: the five binaries whole + walkreal 01, 16, 1b + enemy_accuracy_live.
  Results (2026-11-06 11:45 UTC): five reds triaged on the measured ledgers. Verdicts: every
  red is walk-tuning to the OLD guard behaviour (the 2.0 stops, the glass shield) — no game
  code changed, no [L] loosened, no ignore added. FIXED: 06 (the file's d2 aimed-stand law
  extended to the d1 real-damage walk via a stands param; both sisters green; the binary
  6/1 with the 1 its pre-existing 00agent ignore) and 08's pane red (the cabin fights' ammo
  drift left the clip dry — pane 2's five pulls were clicks; w.reload_full() before the
  shoot-out → 0 end panes glazed). 13: the lobby re-anchor (2.5, 30.5) FIXED the 6000-cap
  (the walk passed the west-mid for the first time) but the ~5000-tick timeline shift moved
  #5's 28.0 into the budget and the room guard #8's point-blank 31.5 killed the walk at
  3006 — the re-anchor-end and doorway sweeps measured byte-identical (rays never open);
  the cap is fixed, the death remains. 0d: the strip-mouth #10 kill dwell 600→180 (the kill
  itself no longer lands — 600 converged ray-clear shots at d 63.6 scored zero, itself a
  look-item) moved the d1 red from the 2627 strip death to a 9300 drop death (the transit
  survives); the bay-sniper class (0x1e at d 18-61 through the bay glass) is unanswered.
  03: the far shooters (d 30-40) outside the plain 20 u window drain the servant/exit legs
  (#11 56.0 in 5 at d 40.2 through glass); the 06 stand_shooters_down law is the proposed
  lever, not landed. 08's station red remains (the branch's own diff predicted it: the
  #23/#19 top-station volleys at d 59-97; death 27596 to a hurt volume after the drain).
  The 2627 death at (208.62,1.04,-229.64) belongs to 0d's walk_arrival_to_the_c5_door_real_
  damage, not 13. Battery (--include-ignored, whole): 06 6/1 (33.3s), 13 2/1 (48.7s),
  0d 1/2 (45.6s), 08 5/1 (194.5s), 03 5/2 (752.2s), 01 8/0 (56.6s); 16/1b/enemy_accuracy
  were still running at report time — their counts are in the report tail if landed.
  Report: docs/reports/guard-repin-master.md.

## guard-repin-master-2
Started 2026-11-06 11:20 UTC. Branch s/guard-repin-master-2 (from origin/s/guard-repin-master,
already up to date with origin/master). Task: the last three reds — 03 real_damage_agent (land
the stand_shooters_down leg-end sweeps from 06 on the servant and exit legs, file-local, d-gated);
13 warm chain real_damage (restore the lobby budget; #8 at the security doorway met Bond at 21 hp);
0d c5 door real_damage and agent (change the "zero-hit while moving" promise to a measured budget,
explain the 600 zero ticks at d 63.6); 08 whole mission real_damage_00agent (station legs re-pin
the declared branch, else measured ignore for the 00 Agent sister only). No [L] loosening beyond
item 3's, no game-code changes to suit a walk. Finish binaries: those whole plus walkreal 01, 06,
16, 1b, enemy_accuracy_live.
## stream-batch (2026-11-06)
Started 2026-11-49 UTC. Branch s/stream-batch cut from origin/s/gas-guard-repin ee3a7dade,
merging origin/s/muzzle-on-master-2 3dbc1a3c7, origin/s/guard-repin-master 0cc4409fd,
origin/s/guard-repin-master-2 c6b6851dc (landed), then origin/master d6245281a (already
contained). Task: one batch re-pin — the gas, guard and muzzle draw families measured and
re-pinned on the JOINED tree so the real-damage walks balance once; keep every gas and
muzzle draw. Merge resolutions (the "most recent tree" rule, per walk): ai_live guard_takes_
cover seed 16 (muzzle tree, 11:09 UTC) over gas's 4 (~03:35 UTC); enemy_accuracy taken
WHOLESALE from guard-repin-master — its harness carries the fixed-stream measure() reset
(Rng::new(0xacd_1b6a), guard-batch-repin's bisect) with its own re-measured pins; the muzzle
branch's no-reset pins (11/11/17 …) ride a different harness model and are superseded by the
joined re-run (the muzzle GAME-code draws stay, nothing lost). walkreal_07000001: kept the gas
lineage's far-field sweep machinery + un-gated lip fights (07:56 UTC, answers the d3 4163
far-pair death; the guard side's poll loop is common ground). walkreal_07000014: kept the gas
side's nudge_west_of_the_door_jamb (the newest tree with a full 26-binary battery on record,
gas-guard-repin 202/23/15). walkreal_07000015: UNION — the guard-lineage skin-based crossing
kill AND the muzzle tree's post-fight nav_until_clear re-clear (different levers, both kept).
walkreal_07000016: kept the gas side's full route economy `true` (a1d200d3c, conditioned on
the gas draws the joined tree carries). docs/WORKLOG.md was add/add (absent on master):
gas-lineage file kept whole, guard-repin-master's entry appended, never rewritten.

## stream-batch-2 (2026-11-06)
Started 14:23 UTC. Branch `s/stream-batch-2` cut from `origin/s/stream-batch` 05ec1cc55,
`origin/master` 38fd3151e merged (clean). Task: the 31 reds of the joined tree's full run
(driver's line on 8e72343f8: 4397 passed / 31 failed / 353 ignored) — non-walk reds first, each
with a stale-pin-vs-merge-break reason; then the walks plain → real damage → Agent → 00 Agent;
the `DroneFunc_HandleExplosives` 0x1435f0 break-after-first-kind-2 review; NF_EXPPROBE removed
from drone.rs; one total line from a full run. Ignore only for 00 Agent walks with a measured
reason; no [L] loosening; no game code changed to suit a walk.
Outcome (15:35 UTC): the HandleExplosives one-record law landed [M 0x1435f0:46 `break` — the
walk stops at the first kind-2-in-radius record pass or fail, ≤1 msg 0x21 per drone per frame;
the port's explosive-outer loop walked on] + NF_EXPPROBE removed. Four non-walk reds fixed with
harness-premise repairs (no game code): grenade_live (the lob's rest moved out of sight on the
joined streams — searched standoff, law asserts unchanged), models_misc (the puff count tracked
every kind-7: per-shot pulses + ambient impact puffs — now the captured puff by born frame),
gun_turrets (the golden moved +8 draws as a block, damage/gaps identical — ambient upstream
consumers, re-pinned with attribution), fix_grenade_types (the flash delivery's CanSeeObject
ray clipped the wall edge on the drifted warm-up — searched offset, law asserts unchanged).
Triage reasons recorded for the other nine non-walk reds (enemy_accuracy's ~2× hit-rate move
needs the deferred bisect; the rest are delivery/route premise shifts — blast spot, plate
route, box approach, firing line, aim premise). Walks: 014's red reproduced tick-exact and the
report's first lever (the muzzle lane-leg) did NOT reach the death (it is on the 1.72-landing
descent above the lane) — reverted; 0d/11 reproduced; the rest not re-run this window. The
full-suite line did not fit (background jobs die between tool calls — verified; the chunk
ceiling cannot hold 03's 752 s): the driver's parent-tree line 4397/31/353 stands as the
reference; this window cleared 4 of the 31 with none added. Report:
docs/reports/stream-batch-2.md.

## stream-batch-3 (2026-11-06)
Started 16:22 UTC. Branch `s/stream-batch-3` cut from `origin/s/stream-batch-2` df294b666,
`origin/master` a6665d115 merged (clean). Task: the 28 reds of the joined tree's full run
(driver's line on stream-batch-2 + a6665d115: 4405 passed / 28 failed / 353 ignored) — each
treated as a possible real regression until shown otherwise, cause named with a citation before
touching any test; then the walks 01, 06, 08, 0b, 0c, 0d, 11, 12, 13, 14, 15, 16 (plain first;
6000-tick-cap failures need a look at what stands there). `goggles_draws_live` run with
`--test-threads=1` before calling it red. Ignore only for 00 Agent walks with a measured reason;
no [L] loosening; no game code changed to suit a walk.
Outcome (17:26 UTC): the merge delta itself is exculpatory [git diff df294b666..9faf17a46 — 3
files +63: the void_fall flag (player.rs, consumed ONLY on 07000012 in scene.rs) + a test], so
the 28 are the batch-2 tree's own. All 10 named non-walk binaries re-run whole (--test-threads=1):
goggles 4/5 with the single-thread run (the captain mode-0 press swallow survives its own 30 u
lever), hold_npcs 1/2 (0.003 park over the full 600 window), guard_drops 5/6 (the deaths' extra
(6,6) is the game's own recovered-sidearm drop [M 0x163350 case 1] — the searched 12-spot blast
lever REFUTED: recovery dominates the frag fuse at every lethal spot; reverted), smg 0/1 (6 u
pistol leg, the unconditional cone-draw stream shift [M Bullet_init 0x12c5a0:171-180]),
enemy_accuracy 4/5 (5 cells, mixed directions = stream position; the law sisters all pass),
upward_shot 0/2 (walks complete, hp 10→10 — the documented realization move), fix_pickups 3/4
(the take + composition + placement-sound asserts pass; the post-take sound window is EMPTY —
needs the box-take sound ledger), start_corridor 7/8 — REAL-REGRESSION SUSPECT: d2's guards
deliver exactly 6×12 (health parks at 28.0) then hold lost_sight=true/seen_ticks=0 for 14000+
ticks at an idle visible Bond (a 20000-tick probe; window restored to 10000, the finding kept in
comments, 424aac6c1). Walks: every PLAIN walk green (01,06,08,0b,0c,11,12,13,14,15,16); the
real-health d2/d3 family dies as its own queue tasks expect; 14 reproduced tick-exact (t5063);
0d real = the fall law holds (18.99 = ~43 pre-fall − 24 game-law damage) with the upstream bleed
moved; 0c real NEW red (t8216 rifle plinks); 13 real NEW failure mode SUSPECT — a void fall to
y −368 at t4169 inside the 11→12→4A chain (the only red touching the merge's 07000012 behavior).
No [L] loosened, no ignore added, no game code changed; two levers tried, one reverted whole.
The full-suite line did not fit again (the batch-2 chunk ceiling stands); every touched-adjacent
binary ran whole with per-binary counts in the report. Live-check asks: the d2 re-see latch, the
deck-sentry stand, the 13-chain shaft transit. Report: docs/reports/stream-batch-3.md.
- void-recovery: branch from origin/master a6665d115; WORKLOG entry started
- void-recovery: the out-of-cel recovery arm shipped — control_handle_cel_change 0x133288's default arm (pos := obj+0x40 clamped into the old cel box minus the 1.0576 world-sphere radius [L ee_start_05 M_CONTROL+0x8C], relink the old cel) on the SP bond, the MP bodies and the poke path; both measured spots reproduced (the (12,6.3,-32) poke returns to the poke origin, the (16.5,6.3,-33) poke clamps to cel 8's max x - 1.0576 = 15.98 vs the measured 15.99); 4 live tests on 07000003 red-proven on the old code; walkreal 03 whole 6/0/1, sisters 02 3/0/1 and 04 10/0, mp_perf_16 2/0; the (16.5) stable-stand residual is the ground-contact model, live-check asked. Report: docs/reports/void-recovery.md.

## level-0a-west-hall (2026-11-06)
Started 18:31 UTC. Branch `s/level-0a-west-hall` cut from `origin/master` d0587e2b5. Task: the
west hall doorway of 0700000a — `walkreal_0700000a_live` ignores two tests where the port
differs from measured game behaviour (`the_west_hall_glide_height_matches_the_game`: settled y
1.0905 on master vs game 1.1285; `the_corner_crosses_like_the_game`). Read both ignore reasons
first (the move-clip idea is dead; the old "crosses at +76 ticks" number came from a dead flag
path). Start from `Player_Collision` 0x194df0 and `Player_CollisionHandler` 0x1949a8; find what
lifts Bond in the doorway — every doorway contact pushes horizontally in the port, so look for
what the game does that the port lacks. Fix + remove the ignores, or report the measured next
lead. Quote walkreal 0a and two other levels' walks.
- (18:31) branch start, WORKLOG entry; reading the ignore reasons next.
