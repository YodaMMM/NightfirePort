
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

## guard-lines-finish
Started 2026-11-06 07:50 UTC. Branch s/guard-lines-finish = origin/s/guard-batch-repin merged with
origin/s/guard-radii-01-d3, origin/s/guard-radii-16-chasers and origin/master (all clean; WORKLOG kept both
lineages + appended). Task: the two non-walk reds from blocked/10-guard-batch-repin.md — (1) enemy_accuracy_live
astronauts_hit_like_the_chain (harness's stream-calibrated expectation; make measure()'s acquisition window
stream-independent), (2) the kiko pit-squad shoot red (kiko_button_real_walk_live.rs:319, reproduced not diagnosed —
find the cause and cite it). Then walks for 01, 16, 1b + enemy_accuracy_live, each binary whole, counts quoted;
d2/d3 ignores on 16 stay unless green on their own.

## guard-lines-finish (results, 2026-11-06 09:35 UTC)
Both non-walk reds fixed HARNESS-side (no game-code change; the glass law stands [M Collide_Update
0x1ea398:216-229]):
1. enemy_accuracy_live astronauts_hit_like_the_chain — `measure()` now resets the shared RNG to a fixed
seed and clears the settle's stray rounds at the window start, so every case measures the fire chain from
the same stream state regardless of what the settle's glass-passed plasma rounds drew. Re-measured pins
(21 cells) on the fixed stream; the rifle bounds now follow the roll law (a rolled miss flies the wobble
bullet and can clip Bond [M 0x179740's miss arm] so hits ≤ rolls; the TimeToHit gate clears early
rolled-true shots [M 0x179740:93-99] so rolled may exceed landed by 4). Whole binary 5/0/4 ignored.
2. kiko pit-squad red — diagnosed by trace + bisect: Bond holds the FISTS (w 0x1) from the tick-0
0x47→0x1 swap; the green master run rode a pit member dying ADJACENT to the shelter door whose drop
(0x14, 21/50) auto-equipped Bond mid-phase (instant swap at tick 9477); the merged fire-path glass pass
shifts the phase's entry state (return-2 leg stalls +545 ticks from the merged tree's guard positions),
no adjacent death happens, the fists' 1.8-u hit bullet [L weapon_data range 1.8] never reaches a member
standing 4.4-5.7 u (its AIM_STAND_FIRE/STEP_AIM standoff), the shoot budget expires untouched. Reverting
ONLY the five glass_pass sites turns the test green (bisect run) — the law is right, the test rode a
drop. Fix: the pit loop cycles the wheel to an owned firearm before every target (real input,
switch_to_armed). Whole binary 1/0 (11.5 s).
Walks whole, counts quoted: walkreal_07000016_live 14/0/2 ignored (the d2/d3 sisters forced-run and
STILL red — kept ignored), walkreal_07000001_live 8/0, walkreal_0700001b_live 17/0/2 ignored,
enemy_accuracy_live 5/0/4, kiko_button_real_walk_live 1/0. The full suite (nf-game/formats/audio + the
remaining binaries) did NOT fit before the 09:28 stop-work mark; left to the next session.
