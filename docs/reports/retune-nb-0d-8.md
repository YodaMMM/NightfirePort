# retune-nb-0d-8 — round 8 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-8`, from origin/master ccd79b5be with origin/s/retune-nb-0d-7 merged in first
(0d-7 was not on master; clean merge 17ad07011). The test `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but this round moved the death 2723 ticks deeper (t19772 → t22495), landed the cleanest
container landing ever measured (34.8 health + 50.0 armour, ZERO landing cost), and measured four new
laws that reshape the round-9 lever. The round-8 lever as written in the 0d-7 ignore message was
refuted on arrival; the census probes that refuted it produced the new route.

## What was done

1. **Death re-measure on the merged tree.** Master moved past the round-7 measurement (ccd79b5be pose
   pool, 434826a9b laser HUD). The merged tree's baseline death is byte-identical to the 0d-6/0d-7
   ledger: t19772 at (141.22, 4.88, -296.08), 21 hits for 80.8 + 19.2 fall, the burst frames
   19676-19682 [L run 1]. Neither master change moved this fight.
2. **THE ROUND-8 LEVER'S PREMISE IS REFUTED — the census (new NF_0D_BAY_CENSUS, prints only, no sim
   state).** At the bay stand (t16104, byte-identical on BOTH timelines — the corridor detour rides
   AFTER the bay stand) the ride killer g18 (owner 18, skin 0x5000087) is IN THE YARD PIT at
   (176.48, 0.98, -282.16) state 0x74, present+enabled, NOT on the bay floor. The in-bay kill box
   (210.0,-288.8) tol [8.0,5.0] found NO live guard within its 14-u census — "no PRESENT guard near —
   already down", landed FALSE on both timelines. The "bay patrol band (207.26,-288.74)->(211.96,-288.73)"
   belongs to g11 (0x5000093) / g22 (0x5000090), both long dead by arrival. And clear_box_aim from the
   bay stand is None for EVERY pit guard (g0/g1/g2/g3/g4/g6/g18 — the yard-east wall): the committed
   in-bay kill never had a target on any timeline [L runs 2-3].
3. **The pit-kill act at the corridor mouth — MEASURED DEAD (one run).** New NF_0D_PIT_KILL=1 (rides
   the corridor roll; unset = the committed round-7 corridor walk byte-identical): after the L1 leg
   the rim act (the head-first down-shot law) burned its 600-tick cap on g18 at (176.90, 0.98, -284.53)
   d 12.4 — clear_box_aim STILL None from (187.89, 10.56, -278.75) [L run 4]. The pit rays are blocked
   from every stand on this route so far.
4. **THE RE-ROLL LAW — the failed act's draws re-rolled the ride to ZERO ride hits.** The pit-kill
   act's 600-tick dwell (plus the detour's own shift) re-rolled the ride: Bond reached the girder
   break at health 34.8 UNTOUCHED — no g18 rounds, no chips (the pit guards shot the GIRDER; the
   committed shift-30 roll took 2 chips, the corridor shift-60 roll took g18's 17.8+17.0) [L runs 4-6].
5. **THE HOP LAW, measured end to end.** The zero-hit roll's hop flew the APEX path: jump fires →
   fall timer 99 → raw 39 × 1.2 = 46.8 fall damage at health 34.8 = death AT the container landing
   (the round-6 dead shape, byte-identical landing (149.26, 8.23, -296.08)) [L run 4]. The sweep:
   NF_0D_HOP_DELAY=30 → identical apex death (girder 31.4, timer 99); NF_0D_HOP_DELAY=60 → ALL hop
   attempts eaten (the crouched idle leaves the jump refused — assert 2114); NF_0D_TAP_CADENCE=17 →
   the apex death again; **NF_0D_TAP_CADENCE=14 → THE CLEAN LANDING: timer 63, raw 3 < 10 = NO pain
   [M 0x1949a8: slti 0xa @0x194bb0], health 34.8 → 34.8 at the container landing, ZERO cost, armour
   50.0 intact — the greenest landing in the eight-round ledger (the ledger's floor sweep needed
   ~28-31; this roll lands 34.8 + 50 armour)** [L runs 5-8].
6. **THE WIRE LAW.** The cadence-14 roll survived the landing and ran the exit: landed standing at
   (141.22, 5.06) health 33.2 — then died t22500 AT the landing spot (crouched sweep posture 4.64).
   The kill shots: "frame 22500: -18.80 from the world (live wire / blast / hurt volume)". The
   floor-stand re-run (new NF_0D_FLOOR_STAND=1, the Operative sister's own measured law — "crouched,
   three kills and Bond left at 4.6; standing, four kills and the yard") skipped the crouch: the wire
   STILL fired standing at 5.06 (-22.40, t22495) — the hazard is PERIODIC (~1300-tick cadence), NOT
   height-gated; the committed rolls never saw it because they died at +622 ticks. The wire's damage
   differs by posture (18.8 crouched / 22.4 standing) [L runs 9-10].
7. **THE FIGHT-POOL CORRECTION.** The 80.8 "invariant pool" is the WHOLE WALK's damage, not the floor
   fight: the cadence-14 roll's 80.8 = #22's pre-ride 46.0 + the wire 22.4 + the floor fight's own
   12.4 (7.2 from #2 at d 4.4 point-blank weapon 0x0 + 5.2 from #1 weapon 0x12). The armour ate the
   rest of the chips as 0.0 deltas (the absorb law [M Player_HandlePain 0x1902c8:84-96]). **Without
   the wire the cadence-14 roll SURVIVES the floor fight: 33.2 - 12.4 = 20.8** [L run 9].
8. **The floor-sweep stall is the round-9 lever.** From the exit landing the first sweep ran ~1345
   ticks with ZERO kill prints (no [0d:dbg]/[0d:ray]/"down at"/cap lines after the landing — the
   stall's exact mechanism is unresolved this round; the pick loop's clear_box_aim and the acts'
   per-tick re-checks are the suspects: the converged garrison hugs the container's south-west corner
   and the container blocks the landing spot's rays — the round-7 law "no clear_box_aim pick remains
   at these rolls' converged positions"). The stall is exactly the wire's output window. ROUND-9
   LEVER: (a) skip/shorten the landing-spot sweep and take the PROVEN-CLEAR stand (142, -291.5) FIRST
   (the stand re-open's own law — the band's rays were probe-read CLEAR standing), sweep from there,
   then walk the yard — the wire never fires if Bond leaves the landing spot inside its ~1300-tick
   cadence; (b) the sweep-stall mechanism needs one audit run (NF_0D_AUDIT + a pass-count print at
   the landing sweep) if (a) alone does not clear it; (c) g18 is still alive in the pit
   (177.07, 0.98, -282.10 at the landing census) but did NOT fire in the floor fight (the hit log's
   owners are #2/#1/#22 + the world) — the pit stays a next-round target only if the yard walk's
   later phases need it.

No guard code changed, no damage/health constants changed, no top-ups: the new machinery is three
env-gated prints/acts (NF_0D_BAY_CENSUS, NF_0D_PIT_KILL, NF_0D_FLOOR_STAND) whose defaults reproduce
the master timeline — the whole-binary run below is the proof of byte-identity.

## How it was verified

Every lever was judged on its own real run through the real-input path (NF_ISO, fresh load, no
teleports after the load). Runs this round (all [L], the direct test binary, `--test-threads=1`,
`--ignored`):

- merged-tree baseline (knobs unset): death t19772 — the 0d-6 byte-identical ledger
- corridor shift-60 re-run (the round-7 config): death t21496 — the round-7 byte-identical ledger
- the corridor shift-60 census run: the bay-stand census at t16104 (g18 in the pit, the box empty)
- the committed-timeline census run: byte-identical census at t16104
- pit-kill run (NF_0D_PIT_KILL=1 + the corridor shift-60 config): the act landed false (the blocked
  ray, d 12.4), the ride re-rolled to zero hits, the apex hop death t21360 (timer 99)
- NF_0D_HOP_DELAY=30: the apex death (girder 31.4, timer 99); NF_0D_HOP_DELAY=60: the all-hops-eaten
  assert (2114); NF_0D_TAP_CADENCE=17: the apex death again
- NF_0D_TAP_CADENCE=14: **the clean landing (timer 63, 34.8 → 34.8, armour 50)** — death t22500 at
  the landing spot (the wire + the stall)
- NF_0D_FLOOR_STAND=1 on top: death t22495 standing (the wire -22.40 confirmed periodic)

Whole binaries on the final tree (knobs unset):

- `walkreal_0700000d_live`: **3 passed / 0 failed / 1 ignored** (99.6 s) — the plain replay, the d1
  real-damage walk and the d2 Agent sister byte-identical.
- `interactables_0700000d_live`: **11 passed / 0 failed** (20.3 s).

The fast runner (`tools/test/suite.py`): started in the background in the first minutes per the
driver's instruction; this sandbox killed it between tool calls (the rounds 1-7 signature —
`suite.out` stayed empty and no suite process survived the first tool call), so the per-binary
foreground runs above are the primary evidence. Crates touched: only `crates/nf-view` (one test
file); no `crates/*/src/` and no `tests/common/` files changed, so no other level's binary is
affected; the two binaries above are the ones the task names.

`cargo test --offline --release -j4 -p nf-view --test walkreal_0700000d_live --no-run` compiles clean
on the final tree.

## What's left

- The walk itself: still red at t22495 — but the round-9 lever is now precise: the floor fight is
  survivable with the landing budget this route already achieves (33.2 + armour vs the floor fight's
  12.4); the wire (periodic ~1300 ticks) and the landing-spot sweep stall are the only killers left.
  Move the sweep to the proven-clear stand (142, -291.5) first, or find and fix the stall, and the
  walk should carry ~20 health into the yard walk proper — the phases the d1/d2 walks already prove
  navigable.
- Live checks for the main agent: none new — the corridor render spots from the round-7 report stand.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for the
session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy failed

The three fallbacks will be attempted after this report is committed; their outcomes are recorded in
the WORKLOG entry if the message fails again (the rounds 1-7 signature is a sandbox-wide network
timeout to ntfy.sh).
