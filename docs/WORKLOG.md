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

## spawn-pick-and-dart-stamps (2026-11-06, s/spawn-pick-and-dart-stamps)

Task: port two drone-AI divergences found reading the decompile, each with a fail-first test:
1. The spawner pick: the game draws Rand_Rand over ALL templates and spawns nothing when the
   drawn template is used or too close [M 0x137cb0:21, 0x137aa0:32]; the port draws
   rand(usable.len()) and always spawns (spawners.rs:194). The port also makes none of the
   game's spawn-chain draws [M 0x14b300:382-461, 0x137cb0:58]. Port both; live PCSX2 wave-frame
   comparison on 0700001b goes to the report as a live check.
2. Darts on the Tower levels: on_dart_impact skips the stamp block the tase runs
   [M 0x1469b8:322-348 vs :89-128]. Port and pin.

Cut from origin/master c4c00c7ec (same tree the civ-unignore tasks ran). Plan: read the four
decompile sites + the port's spawners.rs/drone.rs dart path, write the fail-first tests, port,
then re-pin any walk the new draws shift BY ROUTE (never by removing draws), and run every live
binary that touches spawners.

### spawn-pick-and-dart-stamps progress (2026-11-06, ~03:55 UTC)

- **The spawner pick is ported** [M 0x137cb0:21 + 0x137aa0:31-32, :41-56, :88-96]: one
  Rand_Rand over the WHOLE template list (+0x2E == +0x8, both the census count), drawn BEFORE
  the walk; only usable records decrement; a rank past the last usable spawns NOTHING this
  call. The old rand(usable.len()) always spawned. Fail-first unit tests in the module (the
  modulus assert + the missed-rank assert both failed pre-fix; a_rank_zero passes both models).
- **The spawn-chain draws are ported** [M 0x14b300:382-461 + 0x137cb0:58]: the param-14 class
  switch re-rolls at every spawn (the same block the constructor runs at load — 2 draws, case 4
  draws none), the +0xF4 tail Rand_FRand(0) consumes the stream [M :435], the 0x7000011/14
  engage/combat overrides, and SpawnDrone's Rand_Rand(10000) → obj+0xEC re-rolls the wobble
  (the "unidentified writer" the port's wobble doc carried — it is the spawner path's own stamp).
- **The dart's Tower stamp block is ported** [M 0x1469b8:331-357 — byte-identical to the tase's
  :103-128]: fail-first live pin `the_dart_stamps_the_tower_alert_block_like_the_tase` in
  walkreal_07000009_live.rs (the challenger is sighted; the pre-fix run printed sighted 1,
  alt_mode 14, return_state 0x10, first_attack 1 — all wrong).
- **Walk re-pins by route**: walkreal_07000009's alarm-switch approach now beat-splits and
  accepts the press reach (1.4) — the measured failure was the old single hop WEDGING 3769
  ticks at the switch's 1.08 floor (its "arrival" was the level-exit latch passing the assert
  truthy) while a woken sleeper pressed.
- **Whole-binary results (quoted, --test-threads=1)**: nf-game lib 686/0/5 (incl. the 3 new
  unit tests); walkreal_0700001b_live 17/0/2 (124.16 s — the Drake spawner waves survive the
  new draws); walkreal_07000009_live 9/0/0 (13.72 s); walkreal_0700000a_live 13/0/2 (48.01 s);
  npc_doors_live 6/0/1; fix_guard_misc_live 6/0/0; drake_fight_live 3/0/0;
  walkreal_07000005_live 13/1 — the 00agent 05→06 chain died at t9526 (−35.02,38.49,−43.14,
  13 hits for 100.0): the shifted stream made the 00-fights costlier (40.0 hp at t2174, 23.5 by
  t4321) and the courtyard descent finished him — a real-damage 00 re-pin, not started.
  walkreal_0700000c/0d/12 built but not run in the window.

### spawn-pick-walk-05 started (2026-11-06, ~15:40 UTC)

- Branch `s/spawn-pick-walk-05` = origin/master a6665d115 + merge of origin/s/spawn-pick-and-dart-stamps
  (59072b88c, clean). Task: settle the one red the reviewed branch left — walkreal_07000005_live's
  00agent 05→06 chain — by finding WHY the chain moved; re-route the walk test-side if the new spawn
  rank merely changes who spawns where, fix the change if it is itself wrong (cite the decompile).
- Plan: reproduce the death (t9526, −35.02,38.49,−43.14), trace the shifted stream into 05's
  proximity-spawner attempts, then read the decompile of the spawner pick [M 0x137cb0:21,
  0x137aa0:31-32,41-56,88-96] again on the 05 templates to classify moved-vs-wrong.
- **spawn-pick-walk-05 (2026-11-06, ~16:55 UTC)**: the change re-verified RIGHT against the decompile
  [M 0x137aa0:31-32; 0x137cb0:21,30-47,88-96]; the 05 chain moved on the shifted stream and died to a
  point-blank AUTO-HIT [M 0x179740:113-115 — d < 3.0 = the zero-vector aim] the ditch stand's flanks
  walked Bond into (−60.0 at d 1.1, t1574). Re-pinned test-side: keep_off 3.5 bounds the sniper stands
  (walker.rs's own opt-in, doc cites the same line). walkreal_07000005_live whole: 14/0 (the 00agent
  chain end health 53.2, exit 0x07000006); walkreal_0700001b_live 17/0/2; nf-game lib 688/0/5.
  NEW FINDING: the merge turns walkreal_07000006_live's real_damage + real_damage_agent red (4/2/1)
  — proven green with master's drone.rs+spawners.rs over this tree (the merge's only source files);
  the d1/d2 deaths sit in the searchlight courtyard (the guard-repin fd3f231ab prior art is unmerged).
  The 06 re-pin is the next session's first item. Report: docs/reports/spawn-pick-walk-05.md.
- void-recovery: branch from origin/master a6665d115; WORKLOG entry started
- void-recovery: the out-of-cel recovery arm shipped — control_handle_cel_change 0x133288's default arm (pos := obj+0x40 clamped into the old cel box minus the 1.0576 world-sphere radius [L ee_start_05 M_CONTROL+0x8C], relink the old cel) on the SP bond, the MP bodies and the poke path; both measured spots reproduced (the (12,6.3,-32) poke returns to the poke origin, the (16.5,6.3,-33) poke clamps to cel 8's max x - 1.0576 = 15.98 vs the measured 15.99); 4 live tests on 07000003 red-proven on the old code; walkreal 03 whole 6/0/1, sisters 02 3/0/1 and 04 10/0, mp_perf_16 2/0; the (16.5) stable-stand residual is the ground-contact model, live-check asked. Report: docs/reports/void-recovery.md.

## stream-batch-4
Started 2026-11-06 18:24 UTC. Branch s/stream-batch-4 (from origin/s/stream-batch-3;
origin/s/spawn-pick-walk-05 + origin/master + origin/s/void-recovery merged — the 03
walkreal conflict composed both pre-servant blocks, the door-8 mouth self-gate first then
the door-walk-03b east-lawn saddle, braces rebuilt). Task: the three suspects named with
citations (d2 start-corridor lost_sight latch, 13 void fall, 0c new red), the two measured
test fixes (hold_npcs window 1800, fix_pickups_ammo box-id search), the flashbang drop
count, then stream-batch-3's list, plain walks first.

## stream-batch-5 (2026-11-06)
Started 19:10 UTC. Branch `s/stream-batch-5` cut from `origin/s/stream-batch-4` 95772586e,
`origin/master` ab80ac6ed merged (clean, 3 files +858: zoe_abandon_live, port-map/drones.md,
07000008 VIP-abandon law). Task: void recovery OUT (revert b5e916d84's player.rs/world.rs/
scene.rs changes + drop void_recovery_07000003_live — held, it pushes Bond at valid spots);
re-enable the combat_state re-acquire exit (`if false &&` drone.rs:10900) with the game's own
0x5c shape [M 0x148338:67-70/151/235, 0x164838:26-35]; re-prove the 06/09/16 abseil/door-chase/
bathhouse guards; then the 03 composed blocks, plain walks 01/06/08/0b/0c/11/12/13/14/15/16,
mayhew_hold, fix_npc_doors, enemy_accuracy re-pin, start_corridor d3 owner point; then real
damage / Agent / 00 Agent.
- (19:10) branch start, merge clean, WORKLOG entry.
- (19:20) void recovery OUT: e8b14adb5 reverted clean (player.rs/world.rs/scene.rs, the 4-test
  file dropped); nf-game/nf-view libs compile.
- (19:30) the re-acquire chase: the game's shape read from the decomp — the router sends blind
  guards to 0x5c from three arms [M 0x148338:67-70/151/235], 0x5c re-runs the router [M
  0x164838:26-35], and 0x60 comes ONLY from the no-opponent gate — which the SP FindOpponent
  can never fire while Bond lives [M 0x1424e0:100-121 re-points +0x170 every tick; the CanSee
  clear at :148-153 runs only with no player target]. So the `if false &&` NO_OPPONENT exit is
  NOT the game's shape; removed with the citation trail. The real freeze cause found by probe:
  the port's sight ray is UNscoped (the game scopes guard→Bond LOS from the TARGET's cel
  [M 0x176f18:60-77]); world::line_of_sight_scoped added and seek_opponent_los rewired to it.
- (19:40) the 11 d2 stall diagnosed one layer deeper with the probe (los_probe11): both guards
  park at (−2.34,47.78,22.22) behind a thin column (x −1.63..−1.35, y 47.03..51.12, z ~23.7,
  mat 0x0f + 0x05/0x06 trim patches at y 48.23/48.36), ALL 7 view-bone rays blocked; in 0x5c
  mv=0 with route remaining 4.10 FROZEN for 10000+ ticks — route.follow reports MOVING while
  the body never advances (the creep wedge behind the column). The scoped ray does not clear
  this stall (the column is inside the port's Bond-cel chain); the next lead is the 0x5c
  follow wedge (the game's guard re-positions out of it). [ai:5c] diag added under NF_AILOG.
- (20:15) the battery: walks 09 9/0/0, 16 14/0/2, 06 4/2/1 (the 2 real-damage reds
  baseline-proven pre-existing by rolling my src back), 01 6/2, 08 5/1, 0b 5/2, 0c 2/1/1,
  12 2/1, 13 1/2, 14 7/2/1, 15 5/1/1 — every red a real-health/Agent/00agent leg except the
  two start_corridor reds (6/2, unchanged by my change: the d2 stall = the follow wedge, the
  d3 owner point = batch-4's finding); mayhew_hold 0/1 and fix_npc_doors 0/1 both
  baseline-proven pre-existing; enemy_accuracy did not fit the window. No plain walk
  regressed under the scoped ray.
- (20:35) report: docs/reports/stream-batch-5.md.

## stream-batch-6

Branch `s/stream-batch-6` from `origin/s/stream-joined` (2efbce7a3, stream-batch-5's joined
tree: master + bullet water + the scoped sight ray, the 0x5c re-acquire exit removed). Task:
the failing count doubled from 28 to 59 — bisect the NEW reds first (ai_live, guard_reaction,
guard_side_gates, guard_barks "never reached AIM_STAND_FIRE", spawner_kill "second spawn got
None", anim_events, tase_silent) across stream-batch-5's removal of the re-acquire block/exit
and the spawn-pick merge; then the 0x5c follow wedge (the d2 stall), then non-walk reds, then
the walks (real damage, Agent, 00 Agent; ignore with a measured reason only for 00 Agent walks;
no [L] pin loosening, no walk-driven game code).
- (21:50) BISECT DONE, 7 binaries, --test-threads=1, real ISO. batch-3 659429d32: ALL GREEN
  (37/5/4/1/4/9/2). batch-4 tip 95772586e, batch-5 pre-AI 15e02952c, the joined tip: the SAME
  16 reds — so 116695f5a (the re-acquire exit removal + the scoped ray) is CLEARED. Inside
  batch-4: the spawn-pick merge 369c1b33e red; its ancestry → 29c667a30 (the dart-stamps base)
  GREEN; c4b2d38ce (the spawner pick draws Rand_Rand over the WHOLE template list [M
  0x137cb0:19, :31-56]) turns spawner_kill 2/7 red — **spawner_kill's 7 = c4b2d38ce**;
  f1f37ba47 (the spawn-chain draws [M 0x14b300:382-461] + the dart Tower stamp block [M
  0x1469b8:331-357]) turns ai_live/anim_events/guard_barks/guard_reaction/guard_side_gates/
  tase_silent red — **the other 10 = f1f37ba47**. Both commits' draws VERIFIED against the
  decompile this session (draw counts + order per param-14 class; the pick rank walk; the
  wobble stamp after CreateObj): game-shaped [M]. The red tests pin the PRE-port (draw-less)
  stream — stale pins over the game-correct one; several asserts cite live traces, so re-pins
  need live re-measurement (the driver's [L]-pin rule).
- (21:55) the non-walk reds: abseil (the rope-follow hang 143 vs 144), escort_follow +
  fix_npc_doors (the 01 escort settles at (3.0,6.3,7.3) — one shared advance wedge), goggles
  (the captain laser key stuck mode 0), upward_shot 0c (0/2, batch-3-documented), mayhew_hold +
  fix_npc_doors + start_corridor d2 (batch-3-documented pre-existing) — all red at batch-3
  already per its report except escort_follow/abseil (not in its battery; escort shows the same
  settle point as mayhew/npc_doors' escort wedge).
- (21:56) the walkreal battery (11 binaries, --test-threads=1, real ISO): 03 4/2/2, 04 8/2,
  06 4/2/1, 08 5/1, 0b 6/1, 0c 2/1/1, 0d 1/1/1, 12 2/1, 13 1/2, 14 6/3/1, 15 5/1/1 —
  total 44 passed / 17 failed / 7 ignored. vs batch-5's battery: 0b IMPROVED (5/2 → 6/1, the
  agent leg recovered on the joined tree), 14 gained the tripbomb red (6310fc1d2 pinned it
  9/0+1i on gas-rng-draws-6 — the placed tripbomb not stuck at the retreat, a fight/flush
  shift). The rest match batch-5's reds leg-for-leg (the real-damage/Agent/00agent lane).

## stream-batch-7 — (2026-11-06, UTC)

- (22:05) Branch `s/stream-batch-7` cut from `origin/s/stream-batch-6` (1c1719195), merged
  `origin/master` (e4fb1c512). The merge's 28 conflicts were all one shape: the batch-6 line
  added the grenade `born` field while master added the `loop_handle` field — both kept,
  fields on their own lines; `cargo check --release` clean after the merge.
- Task from the driver: the 16 stale pins (c4b2d38ce, f1f37ba47) get RULE assertions instead of
  stream-position pins (the draws are the game's; a live trace of one run does not make that
  exact value a law) — for each test state the rule from the decompile and what the old pinned
  value was. spawner_kill's 7 tests get fixed to the whole-template draw (which can spawn
  nothing). Then the walk lane: real-damage/Agent/00-Agent walks, 04's stats tests, 14's
  tripbomb wedge, escort_follow, abseil, goggles, upward_shot; ignore with a measured reason
  allowed for 00 Agent walks only. Finish with the still-red list by name.
- (23:50) The stale pins → rules: spawner_kill 9/9 (setup's counting stub was the old
  always-spawn pin; rank-0 stub + the whole-list modulus pins, rule cited per test
  [M 0x137cb0:21/:41-56/:88-96]); guard_reaction path1 (the floor gates the NOTICE
  [M 0x14ac60] — the shot follows the first move's script: step holds through the danim,
  strafe-aim releases on entry); side_gates 4/4 (bring_to_combat window 300 → 1200 move
  cycles); anim_events melee 5/5 (the connect within the 2-tick flight window; the capsule
  contact clamps the first tick at 0.9467/1.5 — identical numbers on the pre-merge tip, so
  the red predates the merge); tase unaware-premise scoping (the stream sight-alerts the
  pocket guard during setup; the msg-7 drop laws assert from any state). Still red: ai_live
  2 (the SG5 burst pin, the UnderCoverAim 90-vs-37 window), tase P2K-hearing (the only
  headroom witness at 0.00 rise — live measurement wanted). The walk lane not reached;
  no ignores added. Full suite did not fit; every named binary ran whole.

## stream-batch-8 (2026-11-07, UTC)

Branch `s/stream-batch-8` off `origin/s/stream-batch-7b` (d725c4855). Plan: merge
`origin/master`, then `origin/s/bullet-pass-water`, then `origin/s/level-gates-world`
without its Player_Move x0.9 change; run the full suite; then fix the listed red set —
non-walk binaries first with a named cause each, then plain walks, real damage, Agent,
00 Agent (ignore with a measured reason allowed for 00 Agent walks only). Checkpoints
pushed as each merge and each fix lands.
## level-gates-world

- (20:12) branch `s/level-gates-world` cut from origin/master c70343b3a. Task: sweep all 127 `GameState._12_4_`-vs-level-code comparisons in the decompile; for my set (Destroy_*, Break_*, Door_*, Car_*, spawners, scripts, level setup, *Talk*) report PORTED (with port location) or MISSING per branch; fix each MISSING with a real-level test proved red on old code.

- (21:55) the sweep landed: 90+ `_12_4_` branches over 40 non-drone functions read and
  classified (report: docs/reports/level-gates-world.md). Three fixes shipped —
  Player_Move's 07000007 ×0.9 party walk lock (red-proved), AttackTalk case 0x11
  (0x93+Rand(2) 1-in-3 on 07000007, walkreal_07000007 6/6 with it), and the inverted
  first-wave spawner stinger removed (the game's gate is waves_completed != 0; the sim site
  drew rand(2) in a phase the game never draws). AttackTalk 5/6 + 7 implemented then
  bisected OUT: the faithful arms shift the 0d/15/16 RNG streams and kill pinned
  real-health legs (evidence + PCSX2 live checks in the report). nf-game lib 688/688;
  every touched level's walkreal green (07 6/6, 0c 3/3+1/1, 0d 2/2, 14 9/9, 15 6/6,
  16 14/14, soldiers_0c 2/2, spawner_kill 9/9). Left: Destroy_Smash's named-target stat
  gate (needs name-table resolution), the View_DrawSky wiring (visual), the 5/6+7
  unblocking via the main agent's debugger.
(stream-batch-8 merge note: the ×0.9 party walk lock above is NOT carried — the merge drops the Player_Move x0.9 change per the task; AttackTalk 0x11 and the stinger fix ARE carried.)

- (00:55, 11-07) The three merges landed and pushed (master clean; bullet-pass-water
  already contained; level-gates-world WITHOUT the Player_Move ×0.9 change — the
  move_locked feature removed whole, AttackTalk 0x11 + the stinger fix carried). Build
  green (1145 test binaries). Suite map on the merged tree, chunked+resumable:
  193/1145 binaries, 739/20/33; all 20 reds match the round's list with their carried
  causes (batch-3/5/6/7), walkreal_07000007 GREEN 6/0 (the level-gates fixes hold), and
  one NEW item: mp_stress_explosives_live killed at exit −9 (OOM-shaped). No ignores
  added. The unmeasured walk lane (01..1b) resumes from the state file.
## aim-object-stage
Started 2026-11-05 ~23:10 UTC. Branch s/aim-object-stage (cut from origin/s/aim-snap-final
23975ae9f). Task: the aim ray still has no object stage, so the game's snap (shutter object
at (139.562, 50.792, -96.706) [L]) and the laser's cut of the 14 wire differ. Add the object
stage with honest glist posing, in the order Collide_Pick 0x1eb320 and Collide_Intersect
0x1ebaa8 walk it. Must end green: interactables_07000014_live, fusebox_geometry_07000014_live,
rocket_glass08_live, rocket_stop_plane08_live, rocket_origin_live, solar_panel_hits_live.
Then run walkreal_07000008, 0c, 14, 16, 1b and report each; the 1b console walk needs firing
spots above the panels. Cite the rule for the rocket's midpoint gate or leave it alone.
Strip the NF_AIMPROBE prints.

Progress log:
- Environment reused; branch pushed; both predecessor reports read.
- The object stage built in aim_snap: sp_bullet_targets candidates, the built-sphere broad
  phase (0x133070's pair), the 0x201 posed-glist refine; the 0x94 & 0x20 skip confirmed in
  the decomp (0x1eb320:141-144) and left unmodeled ([S]).
- Measured: the wire entities' collision glist poses ~0.1-0.3 u behind the 14 stat wall
  face and the models sphere poses 10+ u off — the wires ride the gfx-bounds frame end to
  end (the aim candidate and the flight contact); the panels keep the glist path (the
  walkway pin stays green).
- The doors-in-aim-ray experiment (world.rs dynamic walk, two runs) measured out and was
  reverted: the near-door snap breaks the 14 cut and the calibrated 08 lines. The 08
  shutter snap stays the tip's stat answer pending the obj+0xF0 live check.
- NF_AIMPROBE stripped everywhere (grep 0).
- Final: rocket_glass08 3/0, rocket_stop_plane08 4/0, rocket_origin 1/0,
  solar_panel_hits 1/0 GREEN; fusebox_geometry 0/1 and interactables 9/1 red (the measured
  chain: the near door leaf stops the laser round — the obj+0xF0 proxy + the glist pivot
  are the live checks). Walkreals quoted in the report: 08 5/1, 0c 2/1/1 (the real_damage
  death — my stage vs the tip's unrun sweep, one checkout A/Bs it), 14 6/3/1, 16 13/1/2,
  1b 15/2/2 (console 1 unchanged).
- Done 2026-11-06 ~00:15 UTC.


## beam-weapon-panels-1b
Started 2026-11-05 16:46 UTC. Branch s/beam-weapon-panels-1b (cut from origin/s/panel-mesh-wip 786875a4f;
origin/master fd8ee385b already an ancestor). Task: the start weapon (slot 51, the Beam) breaks panels
49/50 at tick 0 from the 0700001b walkway spot where the real game holds fire 245 frames and breaks
nothing (meter ~95). s/panel-mesh-wip's triangle stage in held_hit_volumes/sp_bullet_targets is right per
[M Collide_Intersect 0x1ebaa8] but leaves the bug and fails walkreal_0700001b at "console 1 latched".
Read the real fire path (bullet? press vs meter? damage/tick? class bits? obj+0xD0 contact?), compare with
the port's instant 20-damage round, fix the console-vs-panel difference, keep solar_panel_hits_live green,
run walkreal_0700001b_live + solar_panels_live. Lead: 0x33 Phoenix bolt 0x100000/0x200000 arms explode at
hit (0x60007c4 Explode_Create); two panels together = blast through the sphere sweep, check whether the
game's blast reaches panel SPs and where the port's first shot lands at tick 0.

## aim-snap-objects
Started 2026-11-05 19:34 UTC. Branch s/aim-snap-objects (cut from origin/s/panel-aim-snap 08ace8edf). Task:
finish the crosshair aim snap — port the OBJECT side of Player_WeaponInitBullet 0x1a4bf0:154-164's
Collide_RayIntersect aiming-point snap (mask 0x20a / 0x208 with +0x6C & 0x800) [L the PCSX2 breakpoints:
the beam round leaves the muzzle y -1.23, rises to -1.09 by x 3.47, clears the panels; rounds crossing a
panel's 4.77 sphere get no hit entry unless they touch its triangles]. Drones::aim_snap covers level+panels
only today; add drones, doors, wires/fuseboxes, breakables, script entities; read Collide_RayIntersect
0x1ec690 + Collide_Pick; the glass mask bit 2 semantics (aim_snap's glass argument may be inverted); check
+0x6C & 0x800 against the records. 13 red on the branch (suite 4386/13): fusebox_geometry_07000014,
interactables_07000014 (the laser cut), rocket_stop_plane08 x2, rocket_glass08 (live-proven — fix the ray),
rocket_origin (update to the snapped point), walkreal_07000014 x2, walkreal_0700000c real_damage,
walkreal_07000016 x2 (check for the same ray gap first), walkreal_0700001b x2 ("console 1 latched" — new
firing spots above the panels; order 3,5,6... stalls at console 8 from (44.77,-8.66,1.68)). Keep
firing_along_the_walkway_breaks_no_panel green; keep the panel-only triangle stage.

## panel-model-glist

Aim snap: finish on the corrected filter. Branch cut from origin/s/aim-snap-placement fc6e21313
(the aim ray's filter reads the placement word — object flags never carry 0x8000, so the old
filter skipped every static surface and the snap never landed). Green on this tip:
firing_along_the_walkway_breaks_no_panel, the ray stopping at x 6.958338 — the game's own point
[L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62),
object 0]. Red on this tip, each to be fixed by matching the game's ray: rocket_glass08_live /
rocket_stop_plane08_live (the rocket must pass the 08 window plane -96.75), the 07000014 laser
cuts (the wire needs to be in the ray — the game's ray hits object glists [L the 0x1c object at
(139.562, 50.792, -96.706), pos (140.719, 49.5, -96.751), r 1.812]), rocket_origin_live against
the true snapped point. Then the walkreal consoles (0c/14/16/1b) from player spots and strip the
NF_AIMPROBE prints.

Progress (panel-model-glist): the flight's object-pass arm shipped — Bullet_CollisionHandler
0x12ce80's obj+0xF0 & 0x60 rule [M disasm 0x12cf58-0x12cf5c; Control_CreateObjEx 0x1343c8:36
writes obj+0xF0 = glist+0x44 | create param] ported as a per-door bullet_pass (the placement
word's 0x60 bits) applied in flush_world_impacts, with the world impact recorded at the
explosion point. rocket_glass08_live 3/0, rocket_stop_plane08_live 4/0, rocket_origin_live 1/0,
fusebox_geometry_07000014_live 1/0 (all whole, quoted); interactables_07000014_live 9/1 (the
east-wing laser cut needs the aim ray's object stage). Report: docs/reports/panel-model-glist.md.

## aim-object-flags
Started 2026-11-06 ~00:15 UTC. Branch s/aim-object-flags = origin/s/aim-object-stage
504ccdf75 + origin/master merged. Task: the 14 security room's near door leaf stops the
laser because Door.bullet_pass reads the placement flags as a stand-in for
obj+0xF0 = glist_record+0x44 | create_param [M Control_CreateObjEx 0x1343c8:36]. Parse
glist+0x44 from the disc; use the true word for the flight's pass rule (andi 0x60 at
0x12cf18) and Collide_PickObj's gate (obj+0xF0 & mask, 0x20 + 0x1020/0x40/0x10 by hittest
flags [M 0x1eae80]). Live anchors: panels f0 0x81, shutter pieces 0xc1, debris 0xa8a0 [L
0700001b audits]. Then fusebox_geometry_07000014_live, interactables_07000014_live, and
every walkreal the earlier reports list red (08, 0c, 14, 16, 1b with firing spots).

Progress log:
- Branch cut + master merged clean; pushed.
- The word sourced: celglist+0x44 = the entity-params payload +4 [M 0x1d1508
  pCurrCelList[0x11] = block+8] = the port's MapObject.flags; glist+8 = the collision
  surface-byte pointer (0 = no collision — the pick's glist requirement). Door.f0 plumbed
  (Door_Create's SetGList path, no create param); bullet_pass = f0 & 0x60. The aim stage
  gained the 0x1eae80 mask gate (0x60 for the aim/bullet hittest 0x20a — both call sites
  build it [M 0x1a4bf0:154, 0x12ef18 t3=522]) + the per-cel walk's f0 & 0x3000 pre-filter
  [M 0x1eb914]. Probe: the 1b whole panel word 0x81 and the ten pieces 0xc1 match the [L]
  audits; every 14 door placement's own word is the bare kind 0xdb (the stand-in read false
  for all of them); the 14 doors resolve the anonymous 'OBJECT_Door' (word 0x0, r 0.4937).
- The 14 blocker re-measured in the port: the near leaf (pl 384, cel 43 = the beam's own
  cel) stops the round on its POSED TRIANGLES at t 0.383; its BUILT SPHERE (centre
  (-89.338,-0.681,31.023), r 0.4937) also catches the beam (sqDist 0.0147 < r² 0.2437) —
  the game's own sphere arm would stop a closed-door shot too; the 05 boot snapshot reads
  f0 0x0 on all five doors [L]. The game-real flow opens the leaf (the walk-in proximity);
  the tests teleport in. Next: the tests walk the real route, or the live check settles
  the door's runtime pose/f0 on 14.
- The mask gate + the cel-linked door stage measured UNLANDABLE alone: either half re-snaps
  the 08 straight shot off the glass pane and the [L] -100 pierce breaks. The proxy's secret
  is out: p.flags & 0x60 reads the kind byte (0xdb & 0x60 = 0x40) — every door passed by
  accident; the true word (the shutters 0x0/0x201) stops the round the game passes, so the
  game's shutter pass rides the node-0x40 flag [M HITDATA+0x51 bit 1, 0x12cea0] — next
  stage. The 14 near leaf measured game-honest: its built sphere catches a closed-door
  beam (sqDist 0.0147 < r^2 0.2437), word 0x0, no door-path f0 writer; the tests teleport
  in with the leaf closed. Report: docs/reports/aim-object-flags.md. Runs: glass08 3/0,
  stop_plane08 4/0, origin 1/0, panel_hits 1/0, fusebox_geometry 0/1, interactables 9/1,
  walkreal_08 5/1 — all equal to the stage tip (the branch is sim-identical, f0 additive).

## aim-node-pass
Started 2026-11-06 ~01:10 UTC. Branch s/aim-node-pass = origin/s/aim-object-flags
af18e350a. Task: (1) verify the 08 shutter nodes' 0x40 flag in the port's shape data,
replace the kind-byte stand-in with Door.f0, let the node rule (HITDATA+0x51 bit 1 [M
0x12cea0-0x12cea8], penetration_roll_node) carry the shutters; (2) land the mask gate +
the cel-linked doors — the new [L]: the 08 rocket's aim ray hits the class-0x1c object at
(139.562, 50.792, -96.706) and the round flies (139.56, -95.472) to (139.568, -100.472)
with that object in its hit chain; (3) rewrite the two 14 fusebox tests to walk the real
route (the leaf open). Must end green: rocket_stop_plane08, rocket_glass08, rocket_origin,
solar_panel_hits, fusebox_geometry_07000014, interactables_07000014; then walkreals 08, 0c,
14, 16, 1b.

Progress log:
- Branch cut + WORKLOG; pushed.
- The node-rule hypothesis FALSIFIED [D]: the 08 shutter leaves' collision bytes are
  {3:52, 4:64, 10:146} (Shutter_1/3) and {16:18} (Shutter_2) — no byte carries 0x40; the
  glist words 0x0/0x201 have no 0x60; the walk's pass sources (the node word, the surface
  byte 0x40/0x80, the glass class, obj+0xF0 & 0x60) are ALL quiet on the shutters, yet [L]
  the round passes with the object in the chain. The stand-in stays; the live check (the
  shutter object's runtime obj+0xF0 + the walked node's +0x51 during a pass shot) goes to
  the report. The walk's loop decoded [M 0x12d800-0x12d810]: a passed node sets state 1
  and continues; the first non-passed node exits with its state-2 verdict.
- The mask gate + the cel-linked doors LANDED in the aim ray; glass08 3/0, stop_plane08
  4/0 green (last session's red was the f0-based bullet_pass, not the snap).
- The fusebox_geometry test walks the player's route: the door stance (the model test's
  dpos+(0,0.6,1.6) pattern), the E press via step_bond's 4th arg opens the leaf (state
  0x8 -> 0x4, standing open at the stance), the walk out to the pinned spot. The cut stays
  red: the beam bends west-up off the wire slab (the posed centre (-89.41, -0.62, 30.89),
  the round crossing at x -89.75) — the snap's posed-glist west shift, the stage report's
  documented pivot question. interactables' rewrite not started (the same blocker).

## aim-node-pass-2
Started 2026-11-06 05:28 UTC. Branch s/aim-node-pass-2 (origin/s/aim-node-pass 3ea60a711 merged with
origin/master — 4 files, clean). Task: finish the 14 cut — rewrite interactables_07000014's cut with the
walk-and-E pattern of the fusebox test, fix the snap per the report's live check, keep the assertions, re-quote
rocket_stop_plane08 / rocket_glass08 / rocket_origin / solar_panel_hits and the walkreal 08 0c 14 16 1b binaries.
No [L] pin loosened.
  Results (2026-11-06 06:57 UTC): the fusebox cut is GREEN — the west-up bend was the test's
  own aim, not the snap: the loop aimed from the EYE (the game's shot ray runs from the HEAD
  [M 0x1a4bf0:152-159], 0.45 lower — the crossing measured y -1.056 vs the slab -0.728..-0.503),
  the aim-mode steering fed the free-aim cursor (saturated [-0.296,+0.4] in 60 ticks, the
  crosshair ~0.228 rad west-up — the measured bend), and the laser's raise script held the view
  pitch through 900 aim-mode ticks. Fix: the head-anchored direct view + hip fire (the 06
  burn_wire pattern); assertions untouched. fusebox_geometry_07000014_live 1/0 (1.76 s). The
  gfx-frame snap (live check 2's fix) is already in aim_snap's wire branch. interactables
  07000014: 9/1 unchanged (the cut rewrite not started — the window went to the diagnosis);
  the fix's three findings are the rewrite's recipe.

## aim-finish
Started 2026-11-06 07:22 UTC. Branch s/aim-finish (from origin/s/aim-node-pass-2 cb62c58e; origin/master c2c4a6e already contained — the merge is a no-op). Task: finish the 14 cut re-quote — rewrite interactables_07000014_live's cut loop the fusebox_geometry way (aim from bond.head(), set yaw/pitch directly, hip-fire plain input; keep every assertion), re-run the aim-stack binaries on the merged tree (fusebox_geometry 14, interactables 14, rocket_stop_plane08, rocket_glass08, rocket_origin, solar_panel_hits, walkreal 08/0c/14/16/1b), and check the two aim filters read inst.placement.flags (not the object flags) with a measured filtered-instance count per level.
- 07:30-08:15: the interactables_07000014 cut rewritten the fusebox_geometry way (the head-anchored direct view, the asin pitch — the old helper's atan(dy/len) under-angled the steep stance shot, the hip fire, the equip waited to weapon==0x4e): 10/0 on the merged tree. The re-quote: the six aim binaries green (fusebox 1/0, interactables 10/0, stop_plane 4/0, glass 3/0, origin 1/0, solar 1/0); the five walkreals against master baselines measured same-session — the aim stack's src REGRESSES 8 tests across 5 levels (08 5/1 vs 6/0, 0c 2/1 vs 3/0, 14 6/3/1 vs 9/0/1, 16 13/1 vs 14/0, 1b 15/2 vs 17/0). The 1b evidence: consoles 1/2 never latch (master: all 8) — the aim snap (absent on master) redirects the console shots; the snap's object stage excludes the script entities per the live-proven 08 lines, so a shot AT an entity snaps past it. The filter check: filter A reads inst.placement.flags (verified + history), filter B reads the object's glist word ([M] obj+0xF0 = glist+0x44 — the game's own gate); per-level counts printed (aim_filter_census). Lesson 9 found live: the four #[cfg(test)] CollisionWorld literals lacked glist_flags — the lib suite did not compile on the aim stack; fixed, lib 24/0 + formats 92/0 + audio 683/0.

## aim-candidates
Started 2026-11-06 08:28 UTC. Branch s/aim-candidates (from origin/s/aim-finish 81616ca, merged origin/master c2c4a6e — clean). Task: the aim snap's object stage stops on objects the game does not have — fix the pre-filter constants (lui s1,0x3000 = 0x30000000, the f0 word glist+0x44|old&0x30008000), add the Collide_PickObj gates (class 5 skip on 0x200, obj+0xfa&8 or glist+8!=0), make the candidate set the per-cel chains + ForcedList, find why sp_bullet_targets returns stream entities, and re-check Door.bullet_pass against the f0 gate under the [L] 08 pins. Quote the six aim binaries + walkreal 08/0c/14/16/1b.
- 09:00-09:45: the hittest settled from the disasm [M ACTION.ELF]: RayIntersect is called with
  the FLAGS word 0x20a/0x208 in the o7 slot (0x1a4d5c-0x1a4d88: addiu t3,0x20a; movn t3,0x208 on
  weapon+0x6c & 0x800; sw $zero,($sp) = the refine arg 0) and the type word +0x94 = 0x201
  [M 0x1ec690's store] — so the PickObj mask is 0x60 (the port's value was right), the flags'
  0x200 class-5 skip is real but vacuous for the port's candidate set, and the obj+0xfa&8 /
  glist+8!=0 gate is active (script entities pass via Script_EntityStart's 0xe4 [M 0x1d8d78:109];
  the shape() requirement is the glist+8 half [M 0x1d1810:23-40]). FIXED: both f0 pre-filter
  sites 0x3000 → 0x30000000 [M Collide_Pick disasm 0x1eb910: lui s1,0x3000]. The chain-membership
  gate (find_cel on the candidate sphere) built and measured a NO-OP on 1b (every candidate
  in-cel — the probe) and was NOT shipped (unverified against the 14 wire legs). Reproduced 1b
  pre-fix (15/2, console 1 unlatched at line 644); post-fix battery: the six aim binaries GREEN
  (fusebox 1/0, interactables_14 10/0, glass 3/0, origin 1/0, stop_plane 4/0, solar 1/0), lib
  683/0, walkreals 08 5/1 / 0c 2/1 / 14 6/3/1 / 16 13/1 / 1b 15/2 — the aim-finish profile
  unchanged: the constants are safe but the 8-walk regression lives in the candidate set
  (step 3), whose discriminator (creation state vs pose vs a kept-apart list) is still open —
  the report carries the evidence and the live checks.

## aim-console-phase
Started 2026-11-06 10:44 UTC. Branch s/aim-console-phase (from origin/s/aim-candidates ceb1ddae8, merged origin/master d6245281a — clean). Task: the 10-aim-candidates premise "the port stops on objects the game does not have" is WITHDRAWN (the PCSX2 rays ran with Bond idle, pre-launch-order, rigs down). Now measured: the port's rig pose matches the game (t 0 (24.44,-8.80,-11.39) both; game frame 1782 (26.25,-3.02,-11.39) vs port t 1620 (26.28,-3.04)); idle only one rig rises; the rest come with the console phase (channel 19, raised by the first wave's death); shoot_consoles starts frame 2746; writing channel 19 alone in PCSX2 raised nothing. No evidence the game's ray skips raised rigs — do NOT remove candidates to make walks pass. Plan: (1) log what the snap stops on at each failing shot of consoles 1/2 (entity, launch-script role, the console's own hit sphere behind it — is a raised rig really in the line?); (2) if so, the snap may be right and the walk's stance wrong — level_station plans from sphere_shot_ok which does not see these entities; make it use the same candidates as the snap, try on 1b; (3) the other six reds (08, 0c, 14 x3, 16) one at a time — aim-stack route levers vs raw-aim tuning, say which is which; (4) for PCSX2 later: what the first wave's death does besides channel 19, in order, with citations. Keep the [L] 08 pins and the 0x30000000 fix. Quote the six aim binaries and walkreal 08/0c/14/16/1b.

## aim-console-phase
Window 2 (2026-11-06 10:44-12:00 UTC). Branch s/aim-console-phase (from origin/s/aim-candidates ceb1ddae8, merged origin/master d6245281a — clean; first entry 10:44).
- 10:50-11:10: the traced snap — aim_snap's body moved into aim_snap_inner with an optional trace sink, aim_snap unchanged, new aim_snap_traced returns (point, Vec<SnapHit>) with one push per winning stop (level/object/wire/door/drone; SnapHit{stage,id,pi,t} in drone.rs). One implementation, zero behavior change — proven by the battery below.
- 11:00-11:25: the 1b probe (walkreal_0700001b_live.rs: the [cands] dump, the [ring] per-bearing scan, the [gate] traces, the [snap] per-shot winners). FINDINGS: (1) console 8's thief is NOT the snap — 120/120 ring bearings snap-clean at t2207 and t4000; the rig pieces (pi30 gate191 script 6000928, hashes 20007e6 r1.80 / 20007f6 r2.70 at z -11.39, ~2 u behind the screens) are candidates only while the rise script streams them (they appear at t1296 con4's winners, gone by t2207) and never beat the screens' own t 0.044; (2) the real failure: the float WEDGES at (44.7,-8.66,1.68) ~2.2 u short of console 8's station (46,-8.12,0) — the bank steps down along x (screens y -0.60/-2.91/-5.21/-7.51 at x 12.7/25.7/38.7/51.8) and the descent overshot onto the room floor where walk-mode pins it, thrusters dead — from there sphere_shot_ok is FALSE (the world clips the bank) and the aim misses 0.54; (3) the pre-merge master delta (770a74927's alert fold) is the only game-code change on the leg — the 1b test file is byte-identical pre/post merge.
- 11:25-11:30: the snap-aware planner wired anyway (the task's step 2, kept as correct-by-evidence): snap_shot_ok (the traced twin on the stance's crosshair ray, the same candidates the snap sees) gates level_station's ring, the default station, and fire; the stall now measures GOAL progress (a wedged crawl moved 0.3 every ~110 ticks and kept moved_at fresh — the stall never tripped); the replan anchors on the console's default station when the wedged spot's straight lines are all bank-clipped; the float grew climb/creep sag guards. Consoles 8/7/1/2 still red (the landing itself not yet prevented — the crossing-waypoint attempt went pathologically slow and was reverted).
- 11:35-11:46: the battery, every number the aim-candidates profile: fusebox_geometry_07000014_live 1/0, interactables_07000014_live 10/0, rocket_stop_plane08_live 4/0, rocket_glass08_live 3/0, rocket_origin_live 1/0, solar_panel_hits_live 1/0; walkreal 08 5/1, 0c 2/1, 14 6/3/1 (single-threaded — 2 threads OOM-killed), 16 13/1, 1b 15/2. The src change is behavior-preserving; the seven reds (08 0c 14x3 16) are the pre-existing aim-finish ones, untouched.
- LEFT: the six reds' per-level triage (the task's step 3) not started beyond 1b; the first wave's death ledger besides channel 19 (step 4) not compiled; 1b's console 8 landing fix (the zero-g volume boundary vs the low stations) open; the full suite not run this window (the battery above + 1b's 15/2 only).

## 1b-zero-g-floor
Started 2026-11-06 12:02 UTC. Branch s/1b-zero-g-floor (from origin/s/aim-console-phase, already
up to date with origin/master). Task: the float wedge on 1b is the WALK's, not the game's (the
driver's correction: with no input PCSX2 and the port settle identically; the pocket under the
bank never ends zero-g by itself [M Player_ZeroG 0x192ab0 has no ChangeSubState]); fix the
comment, find why the walk's thrust cannot leave the pocket (the bank-face thrust or float_to's
lift input), re-run the 1b walks until consoles 7 and 8 latch, and run the [ring]/[gate] probe
on 08, 0c, 14 x3 and 16 to classify snap faults vs route/raw-aim tuning (14's three are the nav
stick at "the corridor south to the leg-2 spot"). No zero-g or collision code changes. Keep the
[L] 08 pins and the 0x30000000 fix.
1b-zero-g-floor OUTCOME (2026-11-06 ~13:15 UTC): the wedge is the walk's (the driver's correction
confirmed) — the [pocket] ray dump maps the enclosure (ceiling 0.83 up = the bank's sloped
underside, riser 4.73 at +x, −x/±z/down clear) and the port's radial push cancels slope-parallel
thrust (frozen ±0.05 through 3000+ ticks of full climb+back; the game's own push tail is dead
[M Intersect_CylGeom's +0x50 guard] and clips instead). Fixed by the two-phase goal (the report's
own next lever): low-station goals cross at the column top goal.y+5 and descend the station's own
column. The wrong "walk-mode pins it, thrusters dead" comment fixed with the [M] citations (the
lift IS actions 7−8 [M 0x192ab0:136-137]). Consoles 7 AND 8 latch — all eight latch
(8@f5511, 7@f5943); the newly-reachable MissileDeploy gate is the next lever (the deadline math
in the report). The probe on 08/0c/14×3/16: ALL route reds, none snap (08 the station hurt
volume; 0c the d 61-90 sniper drain, killers full hp; 14 the leg-2 spot nav stick — the straight
line clips the wall 0.56 u west of the OPEN door (32,2.1,35.8); 16 the point-blank chaser swarm).
Battery: 1b 15/2/2, 08 5/1, 0c 2/1, 14 6/3/1, 16 13/1/2 (all pre-existing counts; the 1b reds now
run 300+ s deeper to the MissileDeploy assert). Report: docs/reports/1b-zero-g-floor.md.

## aim-route-finish
Started 2026-11-06 13:25 UTC. Branch s/aim-route-finish (from origin/s/1b-zero-g-floor, merged
origin/master, clean). Task: finish the aim routes from the 1b-zero-g-floor report — (1) the 1b
missile deadline (consoles 5/8/1/2 land 30-438 ticks late: pre-charge the clip, lower the burst
gate to 36, the 7->1 flight gets the two-phase treatment), (2) the 14 x3 doorway waypoint
(~x 31.8, z 36.3) before the leg-2 spot, (3) the 08/0c/16 route re-pins per the report's
classification. No collision/zero-g/snap changes; the [L] 08 pins and the 0x30000000 fix stay;
zero-g-slide has not landed, so the pocket-avoidance levers stay.
aim-route-finish OUTCOME (2026-11-06 ~14:50 UTC): the 14 x3 route reds LATCH — the leg-2
approach re-anchored on the probe's west lane (29.7, 0.8, 36.2) at both sites; the report's
doorway waypoint (31.8, 36.3) measured WEDGED (the walker's E toggles vs use_door's 0x8
early-return, the leaf's sliding live sphere keeps the kind-2 door cycling — stuck dump 0x8
pos01 0.77, d 1.55, 6000 ticks); whole walkreal_07000014_live 9/0/1 (was 6/3/1). The 1b missile
deadline MEASURED across three drakes runs: the burst gate kept at 48 (36 regressed 5 by +126
and 8 by +348 — the hold-chain recharge churn); the 7→1 departure-side two-phase kept (console
1 36 ticks early, 2 by 24); the anticipation arm added but measured dead (the console scripts
WAIT at holds — best is Some — so the next console's flight runs inside its own hold window;
the next-window lever: count answered holds (3/script [D]) and pre-fly on the last one).
MissileDeploy still [3,3,2,2,3]. 08/0c/16 untouched (the classification carried). The stream
batch: do NOT join yet (the 1b drakes walks must go green first — one stable timeline).
Suite: nf-game/nf-formats/nf-audio --no-fail-fast 162 binaries, 160 ok, disc_golden 11/3 (the
allowed slim-ISO 3), identity_disc = the onfoot sector-count artifact (volume/serial/NTSC/1.00
match); the door-test 4-tuple compile debt (aim-object-flags' missed files) fixed. The rest of
nf-view not run (test-file-only changes: 14 and 1b; no src/, no common/ helper). Report:
docs/reports/aim-route-finish.md.


## aim-route-3
Started 2026-11-06 ~15:36 UTC. Branch s/aim-route-3 (cut from origin/s/aim-route-finish
85ce433f4, merged origin/master a6665d115). Task: (1) 1b: build the aim-route-finish recipe
— count answered holds per console, pre-fly to the next station on the last hold's answer;
measure the estimated 150-tick lateness of 8; goal MissileDeploy clear and both drakes walks
green. (2) 08 x1, 0c x1, 16 x1: the report's levers; 0c finishers sit at d 61-90 — check the
weapon Bond holds before any stand (pistol range 50 vs rifle 150+ [weapon_data +0x1c]).
Test-side only; the [L] 08 pins and the 0x30000000 fix stay. Finish with the aim binaries and
walkreal 08 / 0c / 14 / 16 / 1b whole.

Progress log:
- Environment reset; fresh clone, branch cut, master merged, pushed.

Progress log (continued):
- The recipe built + the deadline instrumentation: `answered[pi]` counts SP_UnPause
  activations (the damage tail's one-tick flag [M 0x1dc2f8-0x1dc318]) = the answered holds;
  the pre-fly to the next un-latched station fires on the 3rd answer; every latch logs
  `deadline = missile_start[k-1] + P0_LEN[k-1] - 1` [M 0x1aa218/0x1aa23c-258; D P0_LEN].
- MEASURED run 1 (the recipe, the 48-burst kept): ch19 = 2747, all 8 missile starts 2747;
  latches 3@4017 (−529) 4@4425 (−121) 5@4629 (+83) 6@5043 (−103) 8@5505 (+359) 7@5961 (−85)
  1@6388 (+342) 2@6531 (+485); MissileDeploy [3,3,2,2,3,2,2,3]. The pre-fly fires on all 8
  consoles and gains ~6 ticks — the flights were already inside the end-plays.
- The chain decomposition (run 1): twin taps 36 t, height re-floats ~78, clip dry gaps 289
  (k5→k6) and 360 (k7 h1→h2), the real flights 336 (6→8) and 353 (7→1); all 8 UI scripts hold
  in parallel from ~S+1009 — the serial flow is the wave cost.
- CHALLENGER 1 (the steady cadence, clip ≥ 16 replaces the 48-burst): MEASURED WORSE
  ([3,3,2,2,3,3,3,3] — k4 −126 but k5 +108, k8 +414, k7 arms): the early chains consumed the
  clip ration the 48-threshold saved for the late ones. REVERTED.
- CHALLENGER 2 (the interleaved fire pick): bit-identical to challenger 1 through k7 — the
  level look pins a tap to the eye's own height band, so the only free targets are the height
  twins, which the serial flow already answers in ~36 ticks. REVERTED. The 48-burst survives.
- Both drakes walks still red at the MissileDeploy gate; the next levers (deadline-rationed
  burst gate, the fight-exit clip top-up, the mid-bank dual stance) are in the report.
- 08 x1 / 0c x1 / 16 x1 NOT started (the 1b item consumed the window) — the report carries
  the classification and the 0c weapon-range hint.
- Suite not run (time): one test file changed (walkreal_0700001b_live.rs), no src/, no
  common/; the drakes walks ran 3× whole.

## aim-route-4
Task: fix the missile countdown (SpaceMissile_Update state 0 loads 0x30d0dc = REC_FRAME_RATE = 1/60 [M 0x1aa1f0-0x1aa1f4], so COUNTDOWN_STEP 512.0 in space_missile.rs is wrong and the seeds are seconds), then re-pace the two drake walks in 1b from the true deadlines (launch + seed x 60 + P0 length) and the console order they imply, then 08 / 0c / 16 with the levers in 1b-zero-g-floor.md section 3. Test-side only; the [L] 08 pins and the 0x30000000 fix stay.

Progress log:
- Branch cut from origin/s/aim-route-3 (f49ded65a); origin/master already an ancestor, merge a no-op. Pushed.

Progress log (aim-route-4 continued):
- The countdown fix shipped (085b88ca7): COUNTDOWN_STEP = REC_FRAME_RATE 1/60 [M 0x1aa1f0-0x1aa208
  disassembled], the p3 seeds are seconds — and the seed map CORRECTED to 105/105/40/20/0/60/105/60
  by index [D placement re-dump; the old list was mis-ordered against p4].
- ch 19 pinned as the AND[193,194] mux's out (the mux writes it every tick — the raw ch-19 poke
  in PCSX2 held one tick, hence "started nothing in 118 s"); the drive tests hold the mux's inputs.
- The drakes walks re-ordered onto the true deadlines [5,4,3,6,8,7,1,2] (k5 +1799 … k1/k2/k7 +9599)
  with a 12000-tick phase cap: MEASURED — all eight latched early (k5 3993 −553 … k2 10803 −1542),
  MissileDeploy all-2 — the gate that killed the last three windows PASSED without the clip levers.
- interactables 9/9, drake_fight 3/3, space_laser 4/4, nf-game space_missile 6/6;
  walkreal_0700001b 15/2/2 — the two drakes walks now red LATER: Drake's fight (run 1) and the
  real-health console starvation on 7/1/2 (run 2). Report: docs/reports/aim-route-4.md.

## aim-route-5

Branch `s/aim-route-5` from `origin/s/aim-route-4` (b991dc45d), merged `origin/master`
(ab80ac6ed — the one-way doors + the escort VIP fail, clean). Scope per the driver: the
console phase is green (aim-route-4 §2); the next reds are Drake's fight and the real-health
clip (§4), then 08/0c/16 with the levers in docs/reports/1b-zero-g-floor.md §3 — for 0c check
what Bond holds first (pistol-class range 50 vs the d 61-90 snipers). Test-side only; the
[L] 08 pins and the 0x30000000 fix stay. Carry-over from the missile-countdown-master window:
on master's RAW aim both drakes walks went green (17/0/2 whole) with the tail [1,2,7], the
tail-only wave defense (the first-wave fight), and the bounded ch-0x09 monitor-read wait —
Drake's fight is green on raw aim, so the aim-side fight red is the snap stack's own shape.

Outcome: the merge (master, tests/docs only) + one walkreal port commit. BOTH aim-route-4 §4
reds resolved on the snap stack: the real-health walk survives the astronaut wave (the tail
[1,2,7] + the tail-only wave defense via the first-wave fight — the same shape master's raw
aim proved), and Drake's fight KILLS — the cause settled: the wave, alive at the fight in the
old shape, stole the fight's 3.5-u always-hit band target picks; with it answered mid-phase
the fight faces Drake alone. walkreal_0700001b_live 17/0/2 whole, drake_fight_live 3/0,
interactables_0700001b_live 9/0, space_laser_1b_live 4/0, nf-game --lib 686/0/5;
08 5/1 and 16 13/1/2 unchanged (their pre-existing single reds; the merge changed no src).
0c re-ran red at the same tick; the weapon check cleared (Bond holds 0x1e, +0x1C = 1200) and
the cause pinned one level deeper: the return-fire stands' converged aimed rounds miss a
STEPPING guard (the level file's own still-gate note records the identical mechanism for the
deck sentry) — the window (95,13) and the unseen-fights opt-ins both measured the identical
13461 death and were reverted. Report: docs/reports/aim-route-5.md.

## aim-route-6

Branch `s/aim-route-6` from `origin/s/aim-route-5` (691b1c87c), merged `origin/master`
(7370525a4 — the water-byte landing comment; master's missile countdown stays). One merge
conflict in walkreal_0700001b_live.rs (comment/declaration hunks both sides had touched);
resolved on the aim-route-5 side — master's substantive test code (the 550-tick ch-0x09 wait,
the revised objective asserts, the wave defense) was already byte-identical in aim-route-5,
only wording differed, and the merged file now equals the aim-route-5 tree. Scope per the
driver: 1b is green; finish the 08 / 0c / 16 walks test-side using the aim-route-5 report's
levers (0c's far-sniper cause pinned: the converged aimed rounds miss a STEPPING guard; 08
and 16 unchanged with their single pre-existing reds), then quote the six aim binaries and
walkreal 08 / 0c / 14 / 16 / 1b whole. The [L] 08 pins and the 0x30000000 fix stay. Not in
the stream batch. Test-side only.

Outcome: 0c and 16 walks GREEN test-side; 08's cause re-pinned one level deeper, the
attempted gate documented. The merge (master, tests/docs only) + 3 test commits. The 0c fix:
a new opt-in `sniper_stands` stand in the shared `DamageLog::stand_and_fire` (the level's own
yard_stand_aimed shape — settle, the aim button held, the CURSOR driven to the effective-aim
position, the tap gated on the aim BIT + the effective error 0.01 + the target's root still
18 ticks) plus the shooters window opened to the snipers' (95, 13) stand-off, riding a
thread-local in common/walkreal0c.rs set BEFORE the prologue legs (the snipers open up INSIDE
walk_spawn_to_the_band_at — a flag block after the return never runs; measured: the first
wiring sat after the prologue and the walk died byte-identical). The fights then answer the
d 61-90 0x1e snipers and one landing round kills an hp-10 guard: #20 killed at 13323 (its old
kill-shot was 13463), the walk completes at 82.6 hp. The 16 fix: the pit-squad band hold
factored (`hold_the_band`) and widened to EVERY live hostile inside 3.0 (the 14656 finishers
were pit-room AR guards #15/#14 at d 1.2-1.3 in Bond's back — the old clone-only hold faced
away), and the member kill runs in bounded `shoot_bounded` slices with the band hold
re-checked between them (the death sat inside the 7300-tick fixed-target loop while the
guards walked point-blank); the finale survives at 6.0 hp. 08: the NF_DMG_TRACE run pinned
the 28756 death's world bucket to Bond's OWN rocket — the kind-44, owner-−1 blast at
(113.6, 14.3, −180.3) d 2.7 → −75.0 at tick 28178 (the Hind's 2.0-gun rounds then chewed the
last 3.0) — and the d3-gated effective-aim corridor gate (the round's first ticks fly along
the view + the eased offsets, up to 0.23 rad off the gated aim direction; 0.6 u lateral at
2.7 u) moved the death to 27975 (a −62.5/−15.5 pair) — the chase's mid-flight curve from a
clear first-12-u corridor remains; the forecast gate (extrapolate the machine's flight-time
position) is the next lever. Whole-binary quotes: walkreal 0c 3/0/1 (was red), 16 14/0/2 (was
13/1/2), 1b 17/0/2, 14 9/0/1, 08 5/1 (the 00agent, unchanged red with the new evidence); the
six aim binaries green (fusebox_geometry_14 1/0, interactables_14 10/0, rocket_glass08 3/0,
rocket_origin 1/0, rocket_stop_plane08 4/0, solar_panel_hits 1/0). No `crates/*/src` file
changed; the [L] 08 pins and the 0x30000000 fix untouched. Report:
docs/reports/aim-route-6.md.

## aim-last-seven

Branch `s/aim-last-seven` from `origin/s/aim-on-master` (24cca34f8 — aim-route-6 merged with
master). Test-side only — no `crates/*/src` file changed; the [L] 08 pins and the 0x30000000
fix untouched.

Outcome: four of the seven reds taken out. The 11 stairwell pin: the snap did NOT move bullet
marks off drawn geometry — 272 decals 0 mid-air BOTH sides of the change; the red was the
sister check's stale premise (the fan's 0x40 pass records hugged the mesh once the game's own
convergence law [M 0x1a4bf0:154-164] aimed every round at the crosshair's hit point ON drawn
geometry: 8 off-surface pass records pre-convergence 0e0cfdd01 [L], 6 in-band after, worst
0.200) — the sister now probes the DECLARED fudge plane (map43#126) itself: the probe point
(21.782, 34.802, 1.415) must answer the collision query with the bare 0x40 face and measure
> HOLE_ON_SURFACE_MAX (2.139 [L]) from the drawn mesh. The 14 wire tests (both
panel_sequence + the walk): one burn cuts BOTH of the box's wires — the slabs coplanar and
adjacent (posed x −89.474..−89.349 / −89.342..−89.148 on the z 30.895 plane [L probe]) and
the aim sweep clips both, one burn rose both words 0x87+0x71 and removed both entities [M
FuseBox_Update 0x1adfe8 → Effect_RemoveSObjs 0x1cf360] — cut_box skips wires whose entity the
game already removed; the box's group words verify the cut. The 01/08 00agent walks took the
task's measured ignores: 01 — the AND(0x11,0x12,0x13) hall gate can't complete, guard #11 (hp
6) never exposes (exposed false through the 10800-tick bound at 3.6-4.7 u [L], ch 153 never
fires, Bond at 82.0 health — the budget is NOT the blocker); 08 — the chase's own mid-flight
curve self-blasts at 27975 (−62.5/−15.5, the effective-aim corridor clear for the first 12 u
[L], the aim-route-6 §3 ledger re-run on this branch) — the forecast gate is the next lever.
The 15 agent walk red stands, measured: the base death at 9422 takes exactly 100.0 (the world
48.0 = the #526 rim blast −41.6 + the pops; #11's launcher −8.4 + grenade −15.0; #13's
grenade −17.5 — the thrown grenades outlive the throwers); four levers measured and reverted
(the spot pre-shoots miss — the guards walk off their spots and the spawner hold keeps them
unspawned at the rim; the threshold proximity sweep kills #11 but the flank dance + #10's
direct rounds died the run EARLIER at 7318; the #526 threshold lob is DEAD — the 0x1a record
has no gravity bit (f6c 0x40050 [L the dump: speed 15.35, range 400, dmg 3.5]) and the
straight round dies on the rim's face, los 0.924 [L]; the wait-and-sweep invites the swarm —
6860 with 18 hits, and a −34.3 point-blank pop at 6669) — the walk needs a structural pre-
throw kill stand (the 0c sniper_stands pattern) or the live-checked throw cycle. Whole
binaries (single-threaded): stairwell_impacts_render_07000011_live 1/0/1, panel_sequence
07000014_live 2/0, walk_07000014_live 2/0, walkreal_07000015_live 5/1/1 (the agent red,
unchanged from the base), walkreal_07000001_live 7/0/1 green, walkreal_07000008_live 5/0/1
green. Report: docs/reports/aim-last-seven.md.

## aim-15-agent (started)
Branch `s/aim-15-agent` cut from `origin/s/aim-on-master-2` (4433 passed, 1 failed, 354 ignored baseline). Task: the one red — `walkreal_07000015_live walk_spawn_to_the_omega_sector_exit_real_damage_agent` — gets the opt-in pre-throw kill stand in `DamageLog::stand_and_fire` (the 0c `sniper_stands` pattern), test-side only. Report: docs/reports/aim-15-agent.md.

## aim-15-agent (done)
The one red — `walkreal_07000015_live walk_spawn_to_the_omega_sector_exit_real_damage_agent` — is green: the opt-in `pre_throw_stands` (the 0c `sniper_stands` pattern minus the aim bit — the settle + the 0.01-gate taps) in the shared `DamageLog::stand_and_fire`, test-side only. The release detector is the first-seen-alive tick (`alive_seen`): it catches the spawner templates AND the channel-held placements (no spawn stamp there — the spawn-stamp-only scan missed the west-room pair), inside a 1200-frame window, visible-only, within the plain shooters' 20 u reach. The d2 run also rides `switch_dry` (the fights dried mid-fight), draws the dropped 0x1a back out at the rim (one blast round drops any hp-20 grenadier where the P2K needs six; the draw needed pressed-edge retries after shoot_mine's fire window), and wins the great-hall throw race with a 12-tick door settle + a `stand_and_fire` pre-shoot at door[50] (#5's grenade was airborne at the kill in run 6, −27.7 the killing hit). All flags behind `difficulty == 2 && !w.tops_up()` — the d1/d3/top-up timelines byte-identical. Whole walkreal family single-threaded: 15 → **6/0/1** (was 5/1/1), 0c → 3/0/1, and every other level walk green (01 7/0/1, 02 3/0/1, 03 6/0/1, 04 10/0, 05 14/0, 06 6/0/1, 07 6/0, 08 5/0/1, 09 8/0, 0a 13/0/2, 0b 7/0, 0c_barrelrow 1/0, 0c_eastroute 1/0, 0d 2/0/1, 11 5/0, 11_route 1/0, 12 3/0, 13 3/0, 14 9/0/1, 16 14/0/2, 1b 17/0/2, 41 15/0, 4a 4/0, 4a_satchel 2/0; the top-up walk_07000015 2/0). The #447 fence-shot assert reads the muzzle-aware cause table (the 0x1a grenade round took the #448 diagonal first; the range + no-body asserts unchanged). Report: docs/reports/aim-15-agent.md.
