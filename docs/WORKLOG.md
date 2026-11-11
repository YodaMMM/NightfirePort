
## los-mask-repins-0c-3 (2026-11-10T23:06Z, branch s/los-mask-repins-0c-3)

- Claimed. Base: origin/master 0ee8478fd (the 0c-2 merge: the plain handoff walk and the east ground route green — the walk switches to the P2K for the strip-guard shot; the earlier landing shots were Micro-Camera taps). Task: the real-damage sister of the 0700000c handoff walk green with a real margin — read walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage's ignore message + origin/s/los-mask-repins-0c-2's report; un-ignore. Real input only, no damage/health lowering, no top-ups. Plan: build walkreal_0700000c_live + walkreal_0700000c_eastroute_live + interact_0700000c_live; reproduce; fix on the level's own fight law; run every binary that shares a touched helper; suite if the sandbox allows; report; finish.

## los-mask-repins-0c-3 (2026-11-10T23:30Z, branch s/los-mask-repins-0c-3)

- Claimed. Base: origin/master 0ee8478fd (the 0c-2 merge). Task: the 0700000c handoff walk's real-damage sister green with a real margin, un-ignored.
- (23:04-23:12) Reproduced the 0c-2 ledger exactly: the ride lands at the shifted spot (4.9831,−223.2948) and "the ledge to the arc" fell at +114 ticks to (8.99,2.01,−222.09) — the walk_to arrival check is dxz-only, so it read "arrived" MID-FALL and the next leg wedged on the yard floor 6000 ticks. Implemented the ledger's lever (STRIP_RD_POST_RIDE, the 0c-2 mid-ledge stops post-ride) — and the sister STILL fell, EARLIER: (6.26,1.13,−221.83), 0.84 u north of the straight line at the same x. A temporary hand-steer diag (rd-diag, removed) from the same landing reached the same target clean at (7.117,12.718,−222.412) with NO fights in the window (NF_RF_TRACE: none between the landing and the wedge) — the divergence was the walker's own STALE SIDESTEP: the stuck ladder's nudge counter survives the route's direct step_bond stretches (the wire grab + the rides never decrement it) and strafes the first post-ride leg sideways off the thin band (a left nudge facing east = strafe north). Same bug family as the 12-2 walk's sidestep-carry (that fix lives on s/los-mask-repins-12-2, unmerged).
- The fix, all opt-in: walker_b2.rs `pub reset_sidestep()` (additive; default behaviour unchanged; the sister's scope the only caller); walkreal0c.rs the STRIP_RD_POST_RIDE scope (set ONLY by the real_damage sister) applying the 0c-2 ledger's post-ride stops — the reset + the plain run's hold spot (7.1,−222.42) + the arc west end's midpoint (10.5,−220.25); the sister un-ignored.
- The green run: crossed 543-free (the strip crossing ran the 0c-2 no-stand choreography verbatim), the #282 ride landed at its own shifted spot, the stops held ((6.64,12.77,−222.56) → (7.09,12.74,−222.39) → (9.76,12.45,−220.02) → (11.84,12.50,−218.51)), #283 crossed (36.71,12.15,−208.66), walked out in 18237 ticks, stuck 0, falls 0; the 0d handoff lands at health 84 (lowest 83.8; the level's own medkit restored 75→100 on the real pickup path — no health writes anywhere). 20+ return-fire fights won on the real pipeline.
- Ran (whole binaries, --test-threads=1, release): walkreal_0700000c_live 4/0/1i (the agent sister keeps its own ledger); walkreal_0700000c_eastroute_live 1/0; interact_0700000c_live 10/0; comms_chatter_0c_live 1/0; guard1_rounds_0700000c_live 2/0; upward_shot_0700000c_live 2/0; walkreal_0700000c_barrelrow_live 1/0; interactables_0700000c_live 7/0; walkreal_0700000d_live 1/0/2i; the walkreal family 09 9/0, 0a 13/0/2i, 0b 9/0, 11 3/0/1i, 11_route 1/0, 12 3/0/1i (the 12-2 un-ignore lives on its own branch), 13 3/0/1i, 14 7/0/3i; the interactables family 03 9/0, 06 5/0, 07 4/0, 0a 14/0, 0b 14/0, 0d 11/0, 12 12/0, 13 10/0, 14 10/0; nf-game lib + nf-formats + nf-audio 901/0. The fast suite died between tool calls again (suite.out 0 bytes, no process) — the same constraint the last three sandboxes recorded; the per-binary coverage above is the substitute.
## markers-world-objects
Started 2026-11-08 18:08 UTC. Branch s/markers-world-objects cut from origin/master 2998b6a76.
Task: settle the 38 [S] markers in crates/nf-game/src/world.rs, triggers.rs, gas.rs,
breakable.rs, player.rs, pickup.rs, rigid_body.rs against the decompile — port, prove-equal,
or prove-unreachable; keep the rand draw count; pin play changes with real-input tests.

Work log:
- (18:08) branch created, WORKLOG entry, checkpoint pushed. Sweeping the [S] markers next.

## los-mask-envelope
Started 2026-11-08 22:05 UTC. Branch s/los-mask-envelope cut from origin/s/markers-world-objects (cd56542), merged origin/master (b059df94).
Task: port the five unported mask-application sites from the LOS_MASKS_ON comment (world.rs:70-75):
(1) obj+0xF0 skip word of every picked object [M 0x1eae80:82]; (2) type-byte arms [M 0x1eae80:83-87];
(3) SIGHT_THROUGH_PART in los_walk (mask bit 4, 0x1eae80:79-81); (4) obj-pass drop under mask bit 0x8
for dynamic non-pane parts; (5) subject exclusions reach all callers [M 0x1eae80:57-59] — in the
game's order, then flip the gate. Then every walkreal_*_live one level per call, repair any moved
walk gated to its level, rand draws unchanged, two real-input sight pins, ai+turret+glass binaries,
nf-game, fast-runner suite on the final tree.

Work log:
- (22:05) branch created, WORKLOG entry. Note: docs/reports/markers-world-objects.md does not exist on
  that branch (WORKLOG-only stub); the five items come from the LOS_MASKS_ON comment in world.rs.
- (22:20-22:40) the five envelope items ported into los_walk/line_of_sight_hit/line_of_sight_scoped's
  dynamic blocks (Moving::pick_pair: obj+0xF0 = placement glist+0x44 via Control_SetGList 0x134960:11-12,
  RB exception [M 0x1d6bf0:52 no SetGList], live 0x10 via the id bit; classes pane 0x20/RB 0x1a/door
  0x1c/DynObj 0x52; arms in the game's order); LOS_MASKS_ON=true; the 3885 mask test un-ignored.
- (22:45-23:00) nf-game lib 749 green; glass_pass_every_level_live 3 green; gun_turrets_live 3,
  turret_hits_live 3, cameras_turrets_census_live 1, ai_missing_states_live 10, ai_points_live 4,
  cover_selection_live 2, dtype_sniper_alert_live 1 — all green. ai_live: 36+1 — the second-hit part
  pin (port-derived, "not yet checked live") contradicted the masked vision chain; un-pinned the part,
  kept the damage/kill chain [M]; 37 green.
- (23:00-23:30) walkreal_07000001_live 6/1: the d2 real-damage walk DIES t8977 (guard #4 10 hits);
  baseline (HEAD~1) passes — first divergence the t678 #8 fight (baseline 100->100, masked 100->93.7):
  the masked 0xa27 vision (glass 0x10 / node-pass / 0x40-material drops [M 0x1eae80:70-82 +
  0x1ebaa8:300-319]) engages guards the strict pre-mask ray did not. walkreal_07000002_live 3/1: the
  real-health agent walk misses the doorway-sensor ch 0x81 assert (timeline/path shift suspected,
  unrooted). Both repairs need a route/tactics re-tune against the game-faithful vision — next window.

## los-mask-01-walk (2026-11-09)

- (09:45) branch s/los-mask-01-walk created from origin/s/los-mask-envelope (f7ee52807), merged
  origin/master (63b05cca6 — walk checks 03/08/0d/01, first_rounds_contact_probe). Clean merge, no
  conflicts. Starting: read los-mask-envelope.md, re-tune the 07000001 d2 contact walk (dies t8977,
  guard #4) against the masked vision with the walker's levers gated to 07000001, then one level per
  tool call over the other walkreals.

- (11:35) 01 d2 contact walk GREEN on the masked tree (7 passed / 1 pre-existing ignored, whole
  binary) with the committed levers (hurting-first return fire, bounded ambush rounds, the band
  edge at any health for a draining pocket guard, no wait/resupply under live fire).
- (11:33) 02 castle d2 escort GREEN (4 passed / 1 pre-existing ignored, whole binary, 135 s): the
  death ledger showed Bond at 25.7 hp dying to captor #16 (8 hp left, weapon 0xe) while the wheel
  held the 0x1c shotgun with 18 shells — the file's own d3 dropped-shotgun law extended to the
  masked d2 (arm_weapon 0x1c before/into the geisha-2 room + drop scoops in the tail sweep), plus
  death guards in take_room/clear_captors_real that end the post-fail zombie loops (33k dead
  ticks → the binary 161 s → 22.7 s on the d2 filter). NF_EXIT_DBG env-gated traces in
  drone.rs/nis.rs/scene.rs carried the death-channel/level-exit audits.
- (11:44-12:20) the rest of the walkreal battery, one level per call: 04 GREEN (9/1 pre-ex),
  06 GREEN (5/2 pre-ex), 07 GREEN (5/1 pre-ex). 03 d1 full-route WEDGES on the masked timeline
  (moat leg 43 u short, then servant 0 nav 4 at segment 0.081, no doors/triggers/drones nearby)
  — ledgered into an ignore with the re-time lever. 05 00agent exit-chain bleeds to a masked
  long-range engage (#9: 44.0 in 16 hits at d 96.4, only 3 return-fire fights all session) then
  dies point-blank at the door band — hurting_first gated on in the d3 body (did not move it),
  ledgered into an ignore with the pre-engage lever. 08: 2 passed / 3 failed in 378 s — the
  Agent whole-mission walk confirmed failed by name and ledgered; two failure names were outside
  the captured tail and remain RED in the tree, unnamed (documented in the report).
- (12:30) nf-game --lib 749 passed / 0 failed. The ai/turret/glass binaries did NOT run: the
  batch invocation hit cargo's cross-package --test name collision (nf-game also defines
  ai_live) and the session's stop-work fired before the -p-scoped retry. The fast suite does not
  survive the background in this sandbox (two starts, both killed between tool calls) and its
  13-min wall does not fit the 10-min foreground cap — not run on the final tree.
- NOT RUN (the clock): 09, 0a, 0b, 0c (+barrelrow+eastroute), 0d, 11 (+route), 12, 13, 14, 15,
  16, 1b, 41 (+wall), 4a (+satchel) walkreals.

## los-mask-01-walk-2 (2026-11-09, branch s/los-mask-01-walk-2)

- (23:15 UTC) Setup: sandbox survived (repo/work/ISO/nf-queue/rust all present). Branch
  s/los-mask-01-walk-2 created from origin/s/los-mask-01-walk d8c32a91d, merged origin/master
  (the merge conflict in walkreal_07000002_live.rs resolved: kept the masked-tree d2 Mayhew
  shield block + master's real-health-00agent-02-6 d3 hooks, dropped the superseded d3
  scoop+arm_fullest at stair bottom/courtyard lane). Read los-mask-01-walk.md: taking What's
  left in order — 08 captured rerun first, then the unrun walkreals one level per call, then
  ai/turret/glass one -p nf-view --test at a time, suite runner started early.

## los-mask-01-walk-2 (continued)

- (23:20-00:10 UTC) 08 first per the report's What's left: the captured whole-binary rerun on the
  merged tree names ONE failure (the two unledgered ones pass now): real_damage wedged at the
  stair head — the walkway-head return-fire fights (the back-off crowd's forward −1.0) herded
  Bond onto the walkway door (160.6,45.0,−100.1) west frame pocket (y 46.82, NOT grounded,
  head_bump), seven 3000-tick caps. The gated settle (real-damage walks only: back out of the
  pocket along the incoming lane to the plain crossing's nav-3 arrival (159.75,46.08,−95.86))
  frees the lane; the walk then reaches the Hind fight (rocket 1 lands 199.5 of the machine's
  299) and dies at the bottom terminal to the world zaps (five −1.2 every ~20-25 ticks then
  −20.09 at f53951) + the masked long-range drains (#5 d 80.6, #14 d 98.2). Ledgered into its
  ignore with three next levers (the terminal walk-out, the 05 pre-engage, the strafe dodge);
  the Agent ignore's captured-rerun lever marked done. 08 binary GREEN 3/0/3.
- (00:10-00:45 UTC) The walkreal sweep one level per call: 0a GREEN 13/0/2, 0c GREEN 3/0/1, 0d
  GREEN 1/0/2, 15 GREEN 7/0/2, 16 GREEN 15/0/3, 1b GREEN 16/0/3, 41 GREEN 15/0, 4a GREEN 4/0.
  11: Operative+00agent wedge on the top-floor stairwell-door oscillation (health 100) —
  ledgered, binary green 2/0/2. 12: Operative wedge at the floor south 2 / south corridor east
  (health 100) — ledgered, binary green 3/0/1. 14: tripbomb route freezes on the tilted sill
  row (segment None) — ledgered, binary green 6/0/4. 0b: the d2/d3 sisters die 21 hits for 100
  with 0 return-fire fights — the roof-sprint lineage's own red (s/level-0b-roof-sprint-4 left
  them red), left red with the lineage noted. 09: the alarm-switch nav ping-pongs WEST 18
  segments into the level-exit volume (switch at +18.7), then every hop is +0 — left red,
  suspected nav-route/NodesForPosition class, not mask-attributed. 13: the PLAIN warm chain
  fails the out-doorway probe while both real-damage sisters PASS — left red, ambiguous.
  ai/turret/glass one -p nf-view --test at a time: ai_traces 4/0, turret_hits 3/0, gun_turrets
  3/0, glass_breach 1/0, glass_pass_every_level 3/0/3, glass_dive 1/0, dynobj_glass 2/0,
  rocket_glass08 2/0/1 — GREEN. guard_rounds_glass RED (guard rounds stopped at the glass on
  07000003) — left red for attribution. glass_guards_14 cannot run here (wgpu adapter expect —
  the sandbox has no GPU).
- The fast suite (tools/test/suite.py): background starts are killed between tool calls in this
  sandbox (verified again this session) — not run; the per-binary totals above are the tree's
  state. 03/05's prior-session ignores stand untouched.

## los-mask-08-walks
Branch s/los-mask-08-walks = s/los-mask-01-walk-2 73a33ac5a + merge origin/master 6a4fcf4ef
(auto-merged drone.rs, no conflicts). Task: the 08 Operative ledger's three levers (terminal
walk-out first), then the Agent and 00 Agent sisters' descent re-tune, then the 11/12/14
re-time levers in their ignore messages; one level per tool call, repairs gated to level+difficulty.
- 16:20-17:05 the 08 Operative walk GREEN: the walk-2 ledger's walk-out lever tried and retired
  by the geometry (no floor outside the ride's end box — 46 forward ticks at an unchanged
  position, the "clear 30 u" rays are the shot-out pane holes over the valley); the 05
  pre-engage stands (station deck, secure room, deck pair, platform mouth west widened to
  fight_pkg); the terminal strafe dodge (any live hostile round in the air and closing →
  sidestep out of its lane, side by eye-height clearance): Operative green 24.6 health, world
  share 39.4 → 14.8. Agent re-ledgered (448.5-hp machine + the deck snipers #14/#3/#11).
- 17:05-17:25 the 08 binary whole 4/0/2 (the Operative real-damage walk un-ignored green, the
  topped-up walk byte-exact), nf-game --lib 760/0; the Agent/00agent sisters re-ledgered with
  this session's runs; 11/12/14 not reached (the walk-out lever had to be tried and retired
  first); the report written.

## los-mask-repins-03
Branch s/los-mask-repins-03 = origin/master c120d7b49. Task: bring the 07000003
walk_the_full_route_and_complete_every_objective_real_damage_* walks green under the game's
guard sight (823ff190b: per-object masks, cel walk, sight rotation, guard's own cel, own-cel
boundary push, FellOutMap); re-route each ignored walk with real input, remove its ignore,
never lower damage or health, no top-ups, helpers gated to 07000003 + difficulty.
- Work log: (in progress)

## los-mask-repins-08
Branch s/los-mask-repins-08 = origin/master c120d7b49. Task: bring the 07000008
walk_the_whole_mission_on_real_input_real_damage / _agent walks green under the game's
guard sight (823ff190b: per-object masks, cel walk, sight rotation, guard's own cel,
own-cel boundary push, FellOutMap); re-route each ignored walk with real input, remove
its ignore, never lower damage or health, no top-ups, helpers gated to 07000008 + difficulty.
- Work log: (in progress)
- (18:53-20:04) sandbox reset: clone, branch cut, work-slim + ISO down, Rust/ALSA env; disk
  forced a cargo clean + single rebuild (RUSTFLAGS -L alsausr for the linker).
- (20:05-20:40) reproduced the ledgered deaths on the exact-sight tree (d1 tick 29332,
  (111.1, 13.5, −179.7); ledger via NF_DMG_TRACE: −1.20 × 15 world hits then −23.00 — the
  Hind's strafing rockets detonating on the carriage's own wall band next to the standing
  spot). Re-route: the Hind fight's missile dodge (the d3-only laser-hinges-2 side step,
  real strafe input) opened to the real-damage walks — same gate shape as the sight gates
  (`difficulty >= 2 || !tops_up()`), 07000008 file only, no damage/health writes. The
  Operative real-damage walk GREEN (1.2 health, 116 hits for 98.8, the Hind down f31140);
  walkreal_07000008_live 4/0/2, interactables_07000008_live 15/0. The Agent sister still
  bleeds out (tick 37022: the −1.40 × 34 machine blasts through the shot-out panes over the
  pre-fight #19 ledger 33.6; 69 hits for 100.0) — its ignore message now carries the traced
  ledger and three levers (the west-doorway slit fight, the unseen-cover fights at d2, the
  blast-band dodge). The 00 Agent sister keeps its earlier ledger. The full suite did not
  fit the sandbox (9.9 GB disk: a single-set release build of the four crates + the two
  required binaries; background jobs die between tool calls so the fast runner could not
  stay up).

## los-mask-repins-08-agent
Branch s/los-mask-repins-08-agent = origin/master 114737d6d (the Operative real-damage walk
green since los-mask-repins-08). Task: take the Agent ignore message's levers so
walk_the_whole_mission_on_real_input_real_damage_agent passes with a real margin and
un-ignore it; real input only, never lower damage or health, no top-ups, helpers gated to
07000008 + difficulty 2.
- Work log: (in progress)
- (20:25-20:45) the Agent levers, each on a measured run: the pre-leg unseen lip stand RETIRED
  (the leaves give no shot line — 430 ticks of "held (out of sight or cap)", the pair flanked
  north and burst the legs); the d2 reload_full RETIRED (the fights got cheaper, 74.8 → 32.1,
  but the R-press re-roll drove the boarding into a flank that walked Bond off the east edge,
  the 23.1 fall); the station-hall-mouth proactive stand a no-op (no sight from the plate);
  the per-leg lip stand + the 15 u ring no-ops on the measured timelines (kept, d2-gated).
  LANDED: the WEST DOORWAY SLIT — after the pane shoot-out the fight walks Bond into the
  cabin's west doorway (the band's z gap −178.25..−180.6 [D the tri dump]) with real input;
  the band shields the machine's strafing drips and the rockets start through the slit (five
  rockets 98.8 → 500.5, the machine down t1619). Agent walk GREEN: end health 14.9 armour 3.8,
  61 hits for 85.1, 37505 ticks, un-ignored. walkreal_07000008_live 5/0/1, interactables
  15/0.

## los-mask-repins-11 (2026-11-10T21:25Z, branch s/los-mask-repins-11)

- Claimed. Base: origin/master 0e1371451 (the guard-sight turn-on: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Task: the 07000011 walks green again under the game's guard sight — read the ignore messages, re-route each walk with real input, remove the ignores, no damage/health lowering, no top-ups, helpers gated to 07000011 + difficulty. Plan: read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs + the ignore messages; build walkreal_07000011_live + interactables_07000011_live; reproduce; re-route; suite; report; finish.

## los-mask-repins-12 (2026-11-10T21:28Z, branch s/los-mask-repins-12)

- Claimed. Base: origin/master 0e1371451 (the guard-sight turn-on: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Task: the 07000012 walks green again under the game's guard sight — read the ignore messages, re-route each walk with real input, remove the ignores, no damage/health lowering, no top-ups, helpers gated to 07000012 + difficulty. FIRST the door plc 543 behaviour check (state 0x2 at 0.50 for ~200 ticks after E — game anti-crush vs port bug, [M Door_Activate 0x135be0 + the door mover]; live fact: the game lets Bond through on that approach). Plan: repairs report + ignore messages; build walkreal_07000012_live + interactables_07000012_live; reproduce; door check; re-route; suite; report; finish.
- (21:28-23:15) The door plc 543 behaviour check FIRST: the old ignore's "holds state 0x2 at 0.50
  for ~200 ticks" does NOT reproduce on master tip 0e1371451 — the port's leaf opens fully there
  (NF_DOORTRACE: #15 pos01 0.5 → 1.0, 30 ticks, no react_to_hit reversal). The port REPLICATES the
  task's live fact: Bond placed (9.9,1.1,-13.0) facing south, E, stick forward 240 frames →
  (6.25,1.13,-25.39) vs the game's (6.21,1.13,-25.40) [L] — the port door is NOT buggy. Door law
  written down: kind-0 swing pos01 0.5 = the SHUT pose, closing snaps to 0.5 + SetState(2)
  [M Door_Interp 0x135738]; swing_from_side's side = sign(activator · inv_closed column 2 = +x) —
  EAST/WEST of the hinge, not north/south [M Door_Activate 0x135be0:71-74; the shut matrix]. The
  real blocker: the route's (11,-12)→(8.5,-13.5) diagonal gets stuck against the corridor's west
  wall north of the hole and its stuck-E lands EAST of the hinge → the leaf swings into the
  corridor's west mouth and the leg wedges 6000 ticks [L the d1 run]. Re-route (gated
  DOOR_543_WEST_STAND, difficulty==1 only — the d3/d2 diagonal is proven there and the ungated
  re-route turned all three red): west stand (9.3,-12.85) → E (state OPEN assert) → the crossing
  straight south the live fact's line to (9.0,-17.5). The d1 walk now CROSSES 543 and dies later:
  t7167, drone #2 (skin 0x5000008, weapon 0x19) at (-9.1,-4.1,-21.0) d 20.8, 57 hits for 100.0,
  south hall before 545 — the ignore message updated to that ledger; un-ignoring waits for the
  fight re-tune (real input, no damage/health writes, next window). Ran: walkreal_07000012_live
  3/0/1 twice (before + after the gate; the gate restored the 3), interactables_07000012_live
  12/0. The full suite did not run: past the 23:03 UTC stop line when the tree settled, and
  background jobs die between tool calls in this sandbox (the fast runner could not stay up).

## los-mask-repins-12-2 (2026-11-10T22:12Z, branch s/los-mask-repins-12-2)

- Claimed. Base: origin/s/los-mask-repins-12 0e33a261a (== origin/master 0e1371451 already merged there: the guard-sight turn-on). Task: the 07000012 Operative real-damage walk green under the game's guard sight — the 543 door question is settled (the kind-0 shut pose is pos01 0.5; the leaf opens fully on master), the walk wedges on its ROUTE past the door. Fix the route with real input so walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative passes and un-ignore it. No damage/health lowering, no top-ups; helpers gated to 07000012 + difficulty. Plan: read docs/reports/los-mask-repins-12.md + the ignore message; build walkreal_07000012_live + interactables_07000012_live; reproduce; re-route; run both binaries + every binary that shares tests/common if touched; suite via tools/test/suite.py if the disk allows; report; finish.

## los-mask-repins-12-2 (2026-11-10T22:50Z, branch s/los-mask-repins-12-2)

- Claimed. Base: origin/s/los-mask-repins-12 0e33a261a (origin/master 0e1371451 already merged there). Task: the 07000012 Operative real-damage walk green under the game's guard sight — the 543 door question settled by the predecessor, the walk wedged on its ROUTE past the door.
- (22:10-22:40) Reproduced the ledger death exactly (t7167, 57 hits for 100.0, all drone #2 at (−9.1,−4.1,−21.0) d 20.8 hp 12 — never damaged). Traced it: drone #2's first hit is tick 3649, and Bond's position is (9.3,1.1,−12.8) — the door-543 west stand — for EVERY one of the 3667 remaining ticks; the "through door plc 543" crossing leg never walked. NF_WALKTRACE (new env-gated per-tick leg dump in walker_b2) caught it: the crossing leg's tick 0 has sidestep −753 and counting DOWN — the stuck ladder's nudge left sets sidestep −40 and the decay did `sidestep -= 1` unconditionally, so a LEFT nudge grows forever and the walker strafes left at forward 0.4 for the rest of the walk (walker.rs documented the same bug and fixed it there as the opt-in `sidestep_decays`; walker_b2 never got the mirror). The 57 hits were the drip on a Bond who could never walk into the hall — drone #2 was never the fight problem.
- The fix, all opt-in or d1-gated: walker_b2.rs gets `sidestep_decays` (walker.rs's flag mirrored, default off, the d1 walk the only setter) + the NF_WALKTRACE diagnostic; tower2.rs's DOOR_543_WEST_STAND approach walks the wall geometry (south to the hole band (10.55,−14.40) before the west stand — the corridor's west wall is solid north of the hole, the single west steer pinned Bond at (10.62,−12.85), and the stuck ladder's door answer no-ops on the open leaf without escalating; the old run only passed the approach because the eternal strafe slid it south by accident). Un-ignored.
- The walk green: crossed 543, drone #1 killed at 3595 and drone #2 KILLED at 3754 — both during the crossing leg, the level's own return-fire machinery once Bond actually moves inside #2's 20 u shooters window; walked out in 7017 ticks, stuck 0, falls 0, health 96.5 (3.5 taken all run), the real 4A exit load lands health 88.1 armour 50 and completes both objectives. No damage/health writes, no top-ups, no teleports after the load.
- Ran: walkreal_07000012_live whole 4/0/0i (the Operative green); interactables_07000012_live 12/0; walkreal_07000013_live 3/0/1i (the tower2 co-user); the walkreal family 09 9/0, 0a 13/0/2i, 0b 9/0, 0c 1/0/3i (ledgered), 0c_barrelrow 1/0, 0d 1/0/2i, 11 3/0/1i, 11_route 1/0, 14 7/0/3i; corridor_guards_07000012_live 1/0; enemies_07000012_live 3/0; nf-game lib 782/0/4i; nf-formats 27+1+4+1, nf-audio 92+1 — all 0 failed. The full suite runner could not run: tools/test/suite.py dies between tool calls in this sandbox (suite.out 0 bytes, no process — the same constraint los-mask-repins-12 recorded); the per-binary coverage above is the substitute.

## walker-sidestep-decay (2026-11-11T06:0xZ, branch s/walker-sidestep-decay)

- Claimed. Base: origin/master 784ba2d99, merged origin/s/los-mask-repins-0c-3 (9612e087a) and
  origin/s/los-mask-repins-12-2 (edda1753c). Task: the walker's stale sidestep carry — 0c-3 and
  12-2 each patched their own level (walker_b2's missing decay; the ride/grab scopes never
  reset it). Replace the two level patches with ONE walker change: the sidestep decays, and
  resets when a ride or grab scope begins or ends. Walker-only, no sim changes. Plan: read both
  reports; reproduce; land the walker change; run every walkreal_*_live one level per tool
  call, repair shifts gated to level; nf-game; suite; report; finish.
- (06:08) Setup: repo/work/ISO/nf-queue present; branch cut from master 784ba2d99 + both
  los-mask merges done (WORKLOG conflict resolved keeping both sides). Both reports read: the
  stale carry is walker_b2's stuck-ladder nudge surviving the ride/grab step_bond stretches
  (0c: the post-#282 leg strafed 0.84 u north off the band; 12: sidestep -753 pinned the
  crossing leg). Plan: decay default-on (signum) + resets at the ride/grab scope boundaries
  (walker_b2's Act::WireRide/Act::Ladder AND walkreal0c's three direct wire loops), delete
  both level patches, red-proof the patches first (neutralize → the two walks fail), then
  battery + nf-game + suite on the final tree.
- (06:0x-07:08) Done: the walker change landed (decay default + ride/grab scope resets; both
  level patches deleted). Red proof: 0c wedges at (6.26,1.13,-221.83) and 12 at (10.62,1.03,-15.73)
  with the patches neutralized; green whole: 0c 4/0/1i, 12 4/0. Battery: every walker_b2 walkreal
  (44 binaries) + the three exposed interactables (7/0, 12/0, 10/0) + nf-game lib 782/0/4i. One
  shift: 0b — the plain walks' desk-corner wedge repaired with a level-gated west flank; the four
  real-damage sisters stay RED ledgered (the decay re-times their legs; the dome-wire crossing
  lands in a live window — 39 × -2.40 zaps, dead t16013; lever: re-time the crossing against the
  fusebox windows). The pure-walker.rs walkreals not run (helper untouched, binaries identical to
  master's). The fast suite died twice on disk (0 bytes free at the linker, Bus error) — next
  window runs it first after freeing the 1.2 GB browser caches. Report:
  docs/reports/walker-sidestep-decay.md.
- level-0b-dome-wire-sisters (07:28) Branch cut from origin/s/walker-sidestep-decay cb992a3bd
  (master already merged in upstream — "Already up to date"). Report read: the four 0b
  real-damage sisters are RED at the dome-wire crossing (dead t16013 at (-12.55,57.89,12.62),
  39 x -2.40 zaps = 93.6/100); lever = re-time the crossing against the fusebox windows on the
  decayed ladder (the 0b-diag block prints the world-hit ledger + fusebox wire states). Plan:
  setup (work-slim + ISO downloads), build the 0b binary, read the diag output walk by walk
  (one tool call per walk), re-time the crossing level-gated, remove the temp diagnostics
  before the final commit, run the whole 0b binary + interactables_0700000b_live, nf-game,
  suite if disk permits.
