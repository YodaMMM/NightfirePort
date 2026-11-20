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
