# retune-nb-0d-7 — round 7 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-7`, from origin/master 7ffabafbc with origin/s/retune-nb-0d-6 merged in (0d-6
was not on master). The test `walk_arrival_to_the_c5_door_real_damage_00agent` stays RED; this round
broke the biggest wall the ledger had (the sealed annex pocket) and left the next lever set up precisely.

## What was done

1. **Death re-measure on the merged tree.** Master changed the guards again (SniperAim's ENTER finds the
   elevator door and sets the alert status like the game, c0e23ff23). The merged tree's death is
   byte-identical to the 0d-6 ledger: t19772 at (141.22, 4.88, -296.08), 21 hits for 80.8 + 19.2 fall,
   the converged 4-chip/6-tick burst at frames 19676-19682 [L run 1]. The SniperAim change did not move
   this fight; every ledger law held.
2. **THE WEST-CORRIDOR PROBE — the pocket is NOT sealed.** The 0d-6 ignore message closed the armour
   lever as "SEALED ... unless the corridor proves open". A new knob-gated probe
   (`NF_0D_CORRIDOR_ARMOUR`, riding the existing `NF_0D_ANNEX_ARMOUR` gate, container roll/d3 only,
   defaults byte-identical) walked the full loop from the bay-enter stand (194.0, 10.6, -282.0): west to
   (187.89, 10.56, -278.75) in 70 ticks, north across the z ~-276.4 wedge line at x ~187 **standing** in
   59 ticks (no wall on the corridor line), the corridor middle, then the pickup approach and a 40-tick
   stand that **LANDED THE TAKE: armour 0.0 -> 50, pickup #2 gone** [L run 3]. The return crosses back
   (straight south stalls at z ~-275.09 — the wedge is directional — the diagonal south-east leg returns
   to the bay stand in 94 ticks). Detour ~2158 ticks. Item (7)'s seal claim is superseded in the ledger.
3. **The corridor rolls (each its own real run).** Shift 30 (the committed band): death t21952 (the same
   80.8+19.2 pool; the +50 spent by the re-rolled ride's chip drain, the burst ate the last 17.0 of
   health). `NF_0D_EXIT_CROUCH=1` on top: identical (t21956). `NF_0D_SWEEP_PASSES=24` (a new pass-cap
   knob, default 12 byte-identical): identical t21952 — the sweep loops already break early (no
   clear_box_aim pick remains at these rolls' converged positions; the burst kills inside the committed
   40-tick land_wait before any pass). Shift 30 + `NF_0D_EXIT_YAW=-2.8`: g18 landed one ride round, the
   landing burst finished it (t21867). **Shift 60: the GREEN SHAPE minus one guard** — the pre-ride
   phases carried health 34.8 + armour 33.8 into the ride (the armour ate the 0x12 chips as 0.0
   health-deltas), and the ride's ONLY killer is g18 (owner 18, skin 0x5000087 — the same guard the
   route's in-bay kill targets): two 17.8/17.0 rounds = exactly the 34.8 ride-entry pool, NOT
   armour-absorbed (a bypass damage type at this weapon/state); `NF_0D_TAP_CADENCE=14` did not move it
   (the ride fire is sight-event-keyed). The in-bay kill attempt landed FALSE in every corridor roll.
4. **The refreshed ledger + the round-8 lever are in the ignore message (item 8)**: kill g18/#18 at the
   bay stand before the ride (a knob-gated act variant — longer act, head act, or the box widened over
   the measured patrol band (207.26,-288.74)->(211.96,-288.73)), then `NF_0D_RIDE_SHIFT=60` + the
   corridor take. With g18 dead the ride has no killer and the landing arrives at ~34.8 health + ~33.8
   armour vs the committed 16.8-18.6.

No guard code changed, no damage/health constants changed, no top-ups: the levers are walk-route and
knob machinery only, all behind env gates whose defaults reproduce the master timeline (the whole-binary
run below is the proof of byte-identity).

## How it was verified

Every lever was judged on its own real run through the real-input path (NF_ISO, fresh load, no
teleports after the load). Runs this round (all [L], the direct test binary, `--ignored --test-threads=1`):

- merged-tree baseline: death t19772 (the 0d-6 byte-identical ledger)
- corridor probe (mode 1): the loop walked, the take landed (+50, pickup #2 gone)
- corridor shift 30: t21952; + EXIT_CROUCH: t21956; + SWEEP_PASSES=24: t21952; + EXIT_YAW=-2.8: t21867
- corridor shift 60 (cadence 15): g18 double-tap, ride death t21496; cadence 14: identical t21513

Whole binaries on the final tree (knobs unset):

- `walkreal_0700000d_live`: **3 passed / 0 failed / 1 ignored** (107.2 s) — the plain replay, the d1
  real-damage walk and the d2 Agent sister byte-identical.
- `interactables_0700000d_live`: **11 passed / 0 failed** (34.6 s).

The fast runner (`tools/test/suite.py`) could not run in this sandbox: background jobs are killed
between tool calls here (the same signature rounds 1-6 recorded; `suite.out` stayed empty every
attempt), and the full 450+-binary suite does not fit the 9.9 G rootfs quota — the base image's ML
venv shares it and ~430 built nf-view test binaries had to be deleted to free the link headroom
(relinking all of them is hours). The per-binary foreground runs above are the primary evidence.

`cargo test --offline --release -j2 -p nf-view --test walkreal_0700000d_live --no-run` compiles clean
on the final tree; no other crates' sources were touched.

## What's left

- The walk is still red. The round-8 lever is in the ignore message: the g18/#18 bay-stand kill
  (the act variant) + `NF_0D_RIDE_SHIFT=60` + the corridor take — the shift-60 roll measured this
  session is one guard away from the green path.
- The straight south corridor crossing stalls (the wedge line is directional); the return uses the
  diagonal leg — a future polish, not on the critical path.
- Live checks for the main agent (a render proof of the corridor, complementing the in-sim walk):
  `--bond 194.0,10.6,-282.0,3.7,0.0` should show the bay-enter stand with the annex floor north-west;
  `--bond 187.6,10.56,-276.4,4.7,0.0` should show the corridor line OPEN north (no wall at x ~187 on
  the wedge line); `--bond 192.8,10.6,-272.0,1.6,0.0` should show pickup #2's spot (gone=true after the
  take in a probe run); `--bond 187.1,10.56,-275.1,3.9,0.0` should show the south stall wall (the
  directional wedge face) that blocks the straight return.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for the
session (no gauge, no billing panel, no API counters are exposed to the agent).
