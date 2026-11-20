## retune-nb-0d-6 (2026-11-19, s/retune-nb-0d-6)

Task: round 6 of the 0d arrival-to-C5-door 00 Agent walk `walk_arrival_to_the_c5_door_real_damage_00agent`.
Branched from origin/master 72a68f828 (origin/s/retune-nb-0d-5 does not exist on the remote; master already
carries the rounds 1-5 shipped tree per the ignore-message ledger). Round-5 remaining bug: the committed
shift-30 roll rides alive (2 ride chips, clean landing, exit 16.8) but the yard-floor converged garrison's
4-chip/6-tick burst outraces the sweep's first kill; the round-5 armour walk-over reshape is measured dead
(the wake law — any pocket visit wakes the west-roof #32's sight cadence onto the climb phases). This round:
the round-6 levers from the ignore message — (A) the g18 ride lottery at the untried knob families
(NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60}, each config its own roll),
(C) a pre-ride corner shave that does NOT re-roll the ride. Helpers stay gated to level 0d + 00 Agent;
real input only, no top-ups, never lower damage or health, never change guard code.

### retune-nb-0d-6 progress

- Branch off master 72a68f828, WORKLOG started (docs/WORKLOG.md does not exist on master; rounds 1-5 entries
  were stripped from master on merge — their ledgers live in the queue done/ files and the test's ignore
  message).
- Lever (A) (the round-6 ride lottery) MEASURED DEAD: eight fresh real runs, every untried knob family
  (NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60}) dies ON THE RIDE — the
  girder health (24.0 at shift 30, 16.8 at shift 60) drains to 0 by the container-top landing (fall timer 99,
  landing pos (149.26, 8.23, -296.08) identical across all eight), the panic is the rim sweep's own guard
  ("the rim sweep must not kill Bond on the top", walkreal_0700000d_live.rs:1925); the 40/45 tap zones fire
  extra taps on the ride and the extra draws re-roll the ride's fire onto the rider — the same dice law the
  round-4 cadence hunt measured, now proven for the zone axis too. The baseline (committed defaults) reproduced
  byte for byte first: death t19772, 21 hits for 80.8, other 19.2, #22 46.0, the floor-landing burst
  (#3 10.8 / #1 9.0 / #2 6.0), 0.0 real-damage test FAILED as committed.
- The driver's fallback levers measured dead: the PICKUP SCAN (NF_0D_PICKUP_SCAN, all 14 placed
  pickups) — NO health pickup exists on 0700000d and none near the yard death; the armour pickups
  sit at (147.00, 21.76, -252.77) [2.1+ u above every route point], the closed pocket, and the
  annex pocket (192.82, 10.79, -272.03); the POST-LANDING IDLE WAIT sweep NF_0D_LAND_WAIT
  {0,10,20} (the committed 40) — all dead, the same 80.8 pool (deaths t19757/t19767/t19682); the
  EXIT-FALL DRIFT STEER NF_0D_EXIT_YAW {-2.356,-2.8,-3.14159} — the landing moves ~1.5-2 u and
  the deaths stretch to t20135-20177, but the same 80.8 lands and the landings cost more
  (11.4/13.2/9.6 vs 16.8). THE INVARIANT: sixteen rolls this round all landed exactly 80.8 guard
  damage + 19.2 fall — the walk needs a REMOVED damage source, not a re-roll.
- The ANNEX ARMOUR lever: pickup #2 is 10 u north of the bay-enter stand at the same floor, but
  the plain nav walk, the climb_stairs hop and the crouch crawl all wedge at z ~-276.4 (a
  waist-above wall on the bay side). The 64-ray segment-fan probe maps the pocket OPEN to the
  west/north-west (r4 dirs 9-14; the r2-3 north mouth) with the r6 west corridor — and the
  committed roof lane's (190.0, 19.65, -268.7) sits DIRECTLY ABOVE the pocket's north mouth.
- ROUND-7 LEVER (in the refreshed ignore message): THE ROOF-EDGE ARMOUR DROP — walk off the
  roof's south edge at (190.0, 19.65, -268.7) into the pocket, land in the 1.91 u touch sphere,
  take the +50 (the ~9 m fall costs the same ~19.2 the committed annex drop already pays —
  REPLACE it, not add), exit through the west corridor (along-length unprobed — probe walk
  first) to the bay enter, rejoin the route. The +50 eats #22's 46.0, the ride's 0x12 draws and
  the landing burst [M Player_HandlePain 0x1902c8:84-96].
- Whole walkreal_0700000d_live on the final tree: 3 passed / 0 failed / 1 ignored (99.7 s) —
  the plain replay, the d1 real-damage walk and the d2 Agent sister byte-identical (all new
  machinery is env-gated, defaults reproduce the master timeline). interactables_0700000d_live:
  11 passed / 0 failed (27.7 s). The fast runner (tools/test/suite.py) was started in the
  background in the first minutes; the sandbox killed it between tool calls every time
  (suite.out stayed empty, no suite process) — the same background-jobs-die signature rounds
  1-5 recorded; the per-binary foreground runs above are the primary evidence.
- The ROUND-7 lever (the roof-edge armour drop) was attempted the same session and is MEASURED
  DEAD IN ONE RUN: the walk-off at (190.0, 19.65, -268.7) never fired (40 forward ticks south-east
  ended still grounded at (191.01, 19.62, -268.85) — the roof extends OVER the pocket; the pocket
  is under the roof/shed footprint, no drop exists into it), and the stray pocket nav then ate
  #17's 48.0 on the roof (t16572, health 54.0 -> 6.0) and died t16779. Pickup #2's pocket is
  SEALED (roof/shed above, the bay wall south, only the unprobed west corridor as a possible
  access) — the armour lever is CLOSED at 00 Agent unless a live render proves the west corridor
  open (the report's --bond spots).
- Whole walkreal_0700000d_live re-run on the final tree (the else-restructure): 3 passed / 0
  failed / 1 ignored (101.9 s) — the committed paths byte-identical with the knob unset.

## retune-nb-0d-7 (2026-11-19, s/retune-nb-0d-7)

Task: round 7 of the 0d arrival-to-C5-door 00 Agent walk `walk_arrival_to_the_c5_door_real_damage_00agent`.
Branched from origin/master 7ffabafbc and merged origin/s/retune-nb-0d-6 (not on master). Master changed the
guards again (SniperAim's ENTER finds the elevator door and sets the alert status like the game, c0e23ff23),
so the death is re-measured on the merged tree first. The 0d-6 ledger: the roof-edge armour drop (the round-6
ignore message's ROUND-7 LEVER) was measured dead in one run — the walk-off never fired, the pocket is sealed
(roof/shed above, the bay wall south, only the unprobed west corridor as possible access), and the stray pocket
nav ate #17's 48.0 on the roof. Helpers stay gated to level 0d + 00 Agent; real input only, no top-ups, never
lower damage or health, never change guard code. Each lever judged on its own real run.

### retune-nb-0d-7 progress
- Round-7 re-measure on the merged tree (master 7ffabafbc + 0d-6): the death is byte-identical to the
  0d-6 ledger (t19772, (141.22,4.88,-296.08), 21 hits for 80.8 + 19.2 fall, the converged 4-chip/6-tick
  burst frames 19676-19682) — master's SniperAim ENTER change did NOT move the fight; the ledger's
  damage-class invariant holds.
- ROUND-7 LEVER (the west-corridor probe, the ignore ledger's own precondition) MEASURED OPEN: the new
  NF_0D_CORRIDOR_ARMOUR probe (gated NF_0D_ANNEX_ARMOUR=1 + NF_0D_CORRIDOR_ARMOUR=1/2, defaults
  byte-identical) walked the full loop from the bay-enter stand — L1 west 70 ticks to (187.89,10.56,-278.75),
  L2 north across the z ~-276.4 wedge line at x ~187 STANDING in 59 ticks (no wall on the corridor line!),
  L3 corridor middle 23 ticks, L4 to the pickup (900-tick cap, stopped 0.96 dxz short), the 40-tick take
  stand LANDED THE +50 (armour 0.0 -> 50, pickup #2 gone [L]), R3/R2 back (the straight south crossing
  stalls at z ~-275.09 — 900-tick cap — but the diagonal R1 south-east leg returns to the bay stand in 94
  ticks). Detour cost ~2158 ticks. THE POCKET IS NOT SEALED: the west corridor is a real route at 00 Agent.
- The corridor roll's fight: death moved t19772 -> t21952 (+2180 ticks = the detour's own cost), the same
  80.8+19.2 pool. The +50 was consumed by the RE-ROLLED ride's chip drain (the detour re-rolls the
  sight-event-keyed ride dice; this roll's ride ate ~48.6 of armour vs the committed roll's 3.6) and the
  landing burst (17.0 health) finished it — the sweep picked NOTHING (no clear_box_aim pick remains at
  this roll's converged positions; NF_0D_SWEEP_PASSES 24 changed nothing, the loops break early), the
  burst kills inside the 40-tick land_wait. NF_0D_EXIT_CROUCH=1 on top: identical (t21956).
- Next: the ride is a per-run lottery at the {30,60}-like alive band — each config its own roll.
- The ride lottery at the corridor (each config its own roll): shift 60 + corridor = the GREEN SHAPE
  minus one guard — the pre-ride phases carried health 34.8 + armour 33.8 INTO the ride (the armour ate
  the 0x12 chips as 0.0 health-deltas: #1 7 chips, #0 2), the ride's ONLY killer is g18 (owner 18, skin
  0x5000087 — the same guard the route's in-bay kill targets): two 17.8/17.0 rounds (health deltas, NOT
  absorbed — a bypass type) = exactly the 34.8 ride-entry pool; NF_0D_TAP_CADENCE=14 identical (the ride
  fire is sight-event-keyed). The in-bay kill attempt landed FALSE in every corridor roll. ROUND-8 LEVER
  (in the refreshed ignore message): kill g18/#18 at the bay stand (the knob-gated act variant), then
  shift 60 + the corridor take — with g18 dead the ride has no killer and the landing arrives ~34.8+33.8.
- The refreshed ledger + the round-8 lever are in the ignore message (item 8); item (7)'s "the pocket is
  SEALED" claim is SUPERSEDED (the corridor IS the access).
- Whole walkreal_0700000d_live on the final tree (knobs unset): 3 passed / 0 failed / 1 ignored (100.8 s)
  — the plain replay, the d1 real-damage walk and the d2 Agent sister byte-identical. The ignored 00
  Agent walk stays red (the ride g18 lottery / the landing burst; the ledger + the next lever are in the
  ignore message). interactables_0700000d_live: 11 passed / 0 failed (34.6 s).
- The fast runner (tools/test/suite.py): this sandbox kills background jobs between tool calls (the
  rounds 1-6 signature; suite.out stayed empty every attempt) — the per-binary foreground runs above are
  the primary evidence. The full 450+-binary suite does not fit the sandbox disk either (the venv's ML
  packages share the 9.9 G rootfs quota; ~430 built nf-view test binaries were deleted to free the link
  headroom, relinking all of them for a suite pass is ~hours).

## Task ID: retune-nb-0d-8
Agent: main
Task: retune-nb-0d-8 — the 0d arrival-to-C5-door 00 Agent walk green again (round 8)

Work Log:
- Branch s/retune-nb-0d-8 cut from origin/master (ccd79b5be) with origin/s/retune-nb-0d-7 merged in
  (0d-7 was not on master; merge 17ad07011, clean). WORKLOG entry pushed at branch creation.
- Round-8 lever from the 0d-7 ignore message: kill g18/#18 at the bay stand before the ride
  (act variant over the measured patrol band), then NF_0D_RIDE_SHIFT=60 + the corridor take.
- Master moved past the round-7 measurement (ccd79b5be pose pool / 434826a9b laser HUD) — the death
  re-measure on the merged tree comes first, per the round-7 procedure.
- Round-8 measured: the lever's premise refuted by the census (g18 in the pit, the in-bay box empty
  on BOTH timelines); the pit-kill from the corridor mouth dead (the ray blocked); the failed act's
  draws re-rolled the ride to ZERO hits; NF_0D_TAP_CADENCE=14 lands the CLEAN container landing
  (timer 63, 34.8 -> 34.8, armour 50 — the greenest landing in the ledger); the exit lands at 33.2
  and the yard-floor fight is survivable but the PERIODIC live-wire hazard (~1300-tick cadence,
  18.8-22.4) + the landing-spot sweep stall (~1345 ticks, zero kill prints) kill at t22495/22500.
- The refreshed ledger + the round-9 lever (the proven-clear stand (142,-291.5) FIRST, then the
  sweep; the stall audit) are in the ignore message item (9). Whole binaries: walkreal 3/0/1
  (99.6 s), interactables 11/0 (20.3 s). Report: docs/reports/retune-nb-0d-8.md.

## Task ID: retune-nb-0d-9
Agent: main
Task: retune-nb-0d-9 — the 0d arrival-to-C5-door 00 Agent walk green again (round 9)

Work Log:
- Branch s/retune-nb-0d-9 cut from origin/master (ccd79b5be) with origin/s/retune-nb-0d-8 merged in
  (0d-8 was not on master; the merge is clean). WORKLOG entry pushed at branch creation.
- Sandbox reused from earlier rounds: repo present, no work/ no ISO no cargo — the setup steps
  (downloads, rust, p7z) are being redone this session. In progress.
- Round-9 lever from the 0d-8 ignore message: skip the landing-spot sweep, take the proven-clear
  stand (142, -291.5) FIRST, sweep from there (NF_0D_STAND_FIRST=1) + the audit.
- Round-9 measured: THE WIRE IS DODGED (lever (a)'s core premise proven) — but the escape leg
  needs the new opt-in walker flag skip_return_fire (walker_b2.rs, default false byte-identical,
  this leg only): run 1 pinned the round-8 stall's mechanism (the walk_to return-fire answer
  holds Bond ON the landing spot — the acts cannot land there — until the periodic wire fires,
  death t22479 position unchanged); run 2 walked out at +70 ticks ZERO chips, the wire never
  fired, the stand sweep killed g2 at +57 — then the g6 pass MISSED (the bullet-dir law: the
  bullet flies bond.pitch x FRAC_PI_2 with an anim-posed muzzle; the cursor-y does not steer it)
  and the pack's drain outraced the kills (death t22767). Run 3 = run 2 (the failed-memory can
  not help a pass that outlives Bond); run 4: a GLOBAL aim lift re-rolled the early phases'
  proven acts (death t1789 — the lift is now a stand-sweep-only parameter); run 5: the scoped
  0.88 lift overshot high (the muzzle was 6.12 that shot), the pass burned 260 ticks for 25.2
  health (death t22891). The refreshed ledger + the round-10 lever (the tap gate's Y-law must
  match the bullet's own dir law — muzzle-pose access or a bond.pitch-driven pre-turn; the
  fallback yard-south run variant) are in the ignore message item (10).
- Runs 6-8 (the lift sweep): lift -0.22 LANDS THE FAR KILLS (run 6: g6+g1 down, run 7: g2+57
  then g6 with the nearest-first restored — the kill race closes to ~5-7 health but the
  ~135-tick passes' drain outruns the rest); run 8 re-measured the round-4 crouch law at the
  stand (the crouched escape cost 10.8 vs run 2's standing ZERO) — the presses stay gated, the
  crouch reverted. The round-10 lever: lift -0.22 + nearest-first + a faster pass (the Y-law
  fix) or the yard-south run variant; the full item (10) is in the ignore message.
- Whole binaries on the final tree (knobs unset): walkreal 3/0/1 (102.1 s), interactables 11/0
  (31.0 s). Report: docs/reports/retune-nb-0d-9.md.

## retune-nb-0d-10 (2026-11-20, s/retune-nb-0d-10)
- Branch off master ccd79b5be with origin/s/retune-nb-0d-9 merged in first (0d-9 was not on
  master; fast-forward). Round 10 of the 0d arrival-to-C5-door 00 Agent real-damage walk
  (walk_arrival_to_the_c5_door_real_damage_00agent, still red): the round-10 lever from the
  ignore message item (10) — fix the stand sweep's Y-law to the bullet's own law (the bullet
  flies bond.pitch x FRAC_PI_2 from the anim-posed muzzle, no cursor-y), so the g6-class passes
  land and the runs 6-7 kill race closes at ~10 health margin. Fallback: the yard-south run
  variant (skip the sweep, skip_return_fire through the plate legs). Work continues below.
- The round-10 lever (the ignore message item (10)) implemented: Sim::planned_shot (scene.rs,
  read-only — the spawn arm's own law [M Player_WeaponInitBullet 0x1a4bf0:226-245]: the head or
  the head↔datum-0 midpoint start, the aim snapped by Collide_RayIntersect mask 0x20a/0x208,
  dir = Normalise(aim − start)) + the stand sweep's bullet-line gate (a bullet_gate param, true
  at the stand sweep call site ONLY): fire when the predicted line's crossing of the guard's
  plane lands inside the body box, the trigger tick carrying turn 0 / look 0 so the ease
  prediction is exact. Runs: 10-a (no corridor env — the roll was lost at the landing burst,
  2.4 at the stand), 10-b (corridor without the pit-kill dwell — g18's stable 17.8+17.0 killed
  the ride), 10-c (the full round-9 shape + the first gate: the closed-loop lift OSCILLATED —
  the aim snap slides along the ground behind the guard as the lift shifts, dy swung
  −1.27..+1.43, no shot fired, drained dry at the stand), 10-d (the body-box gate, fixed lift
  −0.22: g2 down one round +57, g0 down ONE TORSO HIT at d 17.4 part Some(4,0.113) — the gate
  LANDS the far kills — but the waits cost 5.8 + 21.6 of armed-pack drain, death in pass 2 at
  4.0), 10-e (the prediction replacing the ray gate: one shot per ~110-tick fire-script cycle
  — the +54 shot missed g2's head box by 0.02 (the lift −0.22 aimed 0.22 below the box; the
  snap-hit law carries the bullet THROUGH the aim point) — 31.4 → 0 in 164 ticks). MEASURED:
  the fight math is unwinnable (one 0x1e shot per ~110 ticks vs the armed 0x12 drain
  ~0.19-0.28/tick and 4-5 guards).
- The ledger's own fallback implemented: NF_0D_YARD_RUN=1 (default 0 = byte-identical; rides
  NF_0D_STAND_FIRST=1) — skip the stand sweep, keep the zero-cost escape walk, run the yard
  legs with skip_return_fire, skip the en-route fight acts (the trench grunts, the gunner #8).
  Run 10-f: the plate + the trench reached (t23384 — the deepest death this family has
  measured), then the roof shooter #29 (138.9, 9.0, −325.3, d 18.2) + the armed yard's 0x12
  chips drained the 31.4 during the return leg. Still red.
- Whole binaries on the final tree (knobs unset): walkreal_0700000d_live 3/0/1 (103.7 s),
  interactables_0700000d_live 11/0 (30.9 s). Report: docs/reports/retune-nb-0d-10.md.

## retune-nb-0d-11 — 0d arrival-to-C5-door 00 Agent walk green again (round 11)
Branch s/retune-nb-0d-11 from master ccd79b5be, 0d-10 merged (d64d868e2). Re-measuring the
death point on the merged tree; taking the next untried lever from the ignore message of
walk_arrival_to_the_c5_door_real_damage_00agent, skipping every ledger item marked measured
dead. Work continues below.
- The round-11 lever implemented: the exact fire law (centre_gate: the predicted line x dir-128
  intersecting any of the guard's part boxes via nf_game::damage::ray_box [M]) + the turn-loop
  break fix (the gated acts stop burning the full 240 pre-phase ticks with the bit held) + the
  composition (the sweep runs under the yard run with the health floor; the trench certain acts
  take the run legs' drains #25/#26/#29/#1 before the return). Runs 11-a..11-f: the certain kill
  PROVEN (g2 down at +58 ticks, one round [L run 11-d]) but the stand entry budget (~20.6) buys
  exactly ONE kill (the pass cost 12.4 incl. the WIRE -4.56 AT THE STAND — the wire volume covers
  the stand); the yard legs' new gate is #6-class back-fire (the C5-door corner); the trench acts
  never ran (the rolls died before the trench). Still red. The round-12 lever (the wire-dodge
  stand + the escape leg's cost + the north band) is in the ignore message item (12).
- Whole binaries on the final tree (knobs unset): walkreal_0700000d_live 3/0/1 (107.2 s),
  interactables_0700000d_live 11/0 (31.9 s). The suite.py background runner was killed between
  tool calls (suite.out empty, the rounds-1-10 signature) and the four-crate build hit the rootfs
  disk wall (9.9 GB); the stale build dirs pruned. Report: docs/reports/retune-nb-0d-11.md.

## retune-nb-0d-12 (2026-11-20, s/retune-nb-0d-12)

Task: round 12 of the 0d arrival-to-C5-door 00 Agent walk `walk_arrival_to_the_c5_door_real_damage_00agent`.
Branched from origin/master ccd79b5be; merged origin/s/retune-nb-0d-11 (not on master) per the task
message. Round-11 remaining bug and ledger live in docs/reports/retune-nb-0d-11.md and the ignore
message; this round takes the next untried lever from the ledger, skipping levers marked measured dead.
- Round-12 measured: THE "LIVE WIRE" IDENTIFIED — no 0xE4 hurt placements exist in 0700000d at all
  (the census probe: sim.hurts empty, folded_hurt admits nothing); the wire damage is the 0xE1
  oil-barrel pair at (143.30, 4.01, -298.41)/(144.22, 4.01, -298.34), health 20, Destroy_Smash
  blast radius 5.0 (param 5 x0.01) + damage 40 (type byte x10), one propagate per boom, falloff
  40 x (1 - gap/5), gap = d - 1.0576. The landing sits at gap 2.23 (a boom 22.2), the escape
  corridor ~3.3-4.4 (~13), the committed stand ON the edge (gap 4.91 -> 0.7; run 11-d's -4.56
  fully explained). Runs 12-a..12-e: the stand-move lever DEAD in all three variants (the far
  west walk 217 ticks 18.0; the nudge-short stopped by arrive 1.2 INTO the band — THE PAIR
  BOOMED ON CAMERA -4.40 at t22689 while g2 STILL died at +58, the fire law's third proof; the
  nudge-full walk 241 ticks 18.0, boom cost 2.11 at gap 5.79); the pre-turn lever NO-GAIN (the
  walk duration is nav/stuck steering, not the turn drag). THE BINDING CONSTRAINT IS NOW THE
  ESCAPE WALK'S EXPOSURE (~0.106/tick x 167-241 ticks); the round-13 lever (the mid-waypoint
  threading + the land_wait shortening + then the north-band kill) is in the ignore message
  item (13). The merged 0d-11 ignore message carried raw quotes (did not compile) — syntax
  fixed. Whole binaries: this sandbox OOMs the single-process runs (3 GB RAM, ~2.9 GB peak) —
  per-test evidence instead: walkreal 3/0 (+1 census ignored), interactables 11/0, each test
  its own process. Report: docs/reports/retune-nb-0d-12.md.

## retune-nb-0d-13

2026-11-20. Branch s/retune-nb-0d-13. Worker round-13 on the 0d arrival-to-C5-door 00 Agent
real-damage walk (walk_arrival_to_the_c5_door_real_damage_00agent).
Branched from origin/master ccd79b5be; merged origin/s/retune-nb-0d-12 (not on master) per the
task message. Round-13 takes the next untried lever from the ignore message item (13), skipping
every lever the ledger marks measured dead; helpers stay gated to the level and difficulty.
- Round-13 measured (five real runs, each its own roll): THE ESCAPE-WALK DRAG IS THE ARRIVAL
  ZONE, NOT THE NAV FIGHTS — the NF_WALKTRACE pinned it [L run 13-d]: ~10 ticks turning, ~12 at
  full stick, then ~160 crawling the walker's arrival zone (d < 2.2 -> forward 0.45, ~0.01
  u/tick). THE WAYPOINT THREAD lever measured WORSE (the per-leg overhead: 379 ticks, the stand
  at 9.8). THE FAST-ARRIVAL LEG PROVEN: the test-local fast_leg (the stick at 1.0 to the arrive
  check, the walker's own tick) walks the escape leg in 105 ticks at cost 3.4 (the stand at
  31.4 vs 15.2) and generalises to the yard legs (NF_0D_YARD_FAST, Legs 2-4 at 110-122 ticks) —
  the round-12 'binding constraint' (the escape walk's exposure) LIFTED. THE LAND_WAIT LEVER
  PROVEN: NF_0D_LAND_WAIT=10 keeps the landing at 34.8 (the 40-tick wait's burst ate armour
  50->0 + 3.4; the 10-tick wait ate 0.2). THE NORTH-BAND KILL implemented (NF_0D_NORTH_KILL)
  and measured UNECONOMICAL (the act's aim phase at d ~17-21 burns ~15-25 under the converged
  pack; the 13-c death mid-act). THE NEW BINDING CONSTRAINT: the armed pack's steady
  sight-event-keyed chips (~0.06-0.10/tick across the legs + acts) exceed the ~31 entry budget
  unless the pack thins — the 13-e sweep pass 0's act refused (the aim-gate roll) and the pack
  drained 31.4->0 in 61 ticks; the 13-f pure-run died at the plate (#29's roof 2.6); the 13-g
  sweep-1-pass died to #6's north-band back-fire. THE ROUND-14 LEVER: THE CROUCHED STAND ACTS
  (the crouch drops the sight target 0.45 [M Player_GetHeadPos 0x195aa0] and the visibility
  x0.75 [M Drone_ProcessOpponents 0x1390c0] — the round-9 run 8's crouched drain ~0.1/tick vs
  ~0.25 standing; the acts' costs halve; never on a walk gait). Whole binaries on the
  knobs-unset tree (this sandbox holds 4 GB and ran them whole, unlike round-12's): walkreal
  3 passed / 0 failed / 2 ignored (102.0 s), interactables 11 passed / 0 failed (31.4 s). The
  suite.py fast runner was killed between tool calls twice (nohup and setsid, suite.out empty —
  the rounds-1-10 signature); the four-crate build also exceeds the rootfs left. Report:
  docs/reports/retune-nb-0d-13.md.
- Round-14 lever TRIED SAME-SESSION (post-finish, four real runs 14-a..14-d): the pass-0
  certain kill landed at 4.2 (the roll; the 1-tick crouch press did not stick — the trench act
  printed the standing eye); SWEEP_PASSES=1 + the fast legs reached THE TRENCH at ~27 (the legs
  66/195/117 ticks) and died mid roof-act — the trench->roof aim never converged (cd(snap) 2.086
  at +330; the roof edge blocks the up-line); the 5-tick press DID crouch but the crouched
  up-pitch aim DEGENERATED (cd(snap) 248) — the crouch is LEVEL-pitch only; the plate roof act
  (NF_0D_PLATE_ROOF_KILL) rolled into the #6 back-fire death. The next-round shape stands: the
  fast legs + the land_wait=10 + SWEEP_PASSES=1 reach the trench at ~25-27; the roof act needs a
  working line; #6's back-fire remains the legs' tax. Final-tree evidence re-verified whole:
  walkreal 3 passed / 0 failed / 2 ignored (106.1 s), interactables 11 passed / 0 failed (31.6 s).
