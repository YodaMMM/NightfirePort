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
