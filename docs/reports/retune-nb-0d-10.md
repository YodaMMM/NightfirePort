# retune-nb-0d-10 — round 10 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-10`, from origin/master ccd79b5be with origin/s/retune-nb-0d-9 merged in
first (0d-9 was not on master; fast-forward). The walk `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but this round built the ledger item (10)'s Y-law fix (the read-only spawn predictor
`Sim::planned_shot` + the stand sweep's bullet-line gate) and PROVED with it that the gate lands
the far kills (g0: one torso hit at d 17.4, part Some(4,0.113)), measured the snap-hit law (the
bullet passes THROUGH the snapped aim point — the runs 6-7 lift was compensating the wrong
branch), measured the stand fight's true budget law (one 0x1e shot per ~110-tick fire-script
cycle vs the armed pack's ~0.19-0.28/tick drain — unwinnable as a fight), and drove the walk to
its deepest death yet (t23384, the display trench reached, via the ledger's own fallback: the
yard-south RUN variant). Six real runs this round; the refreshed ledger and the round-11 lever
are in the ignore message item (11).

## What was done

1. **The spawn predictor** — `Sim::planned_shot(aiming)` (crates/nf-view/src/scene.rs, READ-ONLY,
   no draws, no state writes): replicates the port's own spawn arm exactly — [M
   Player_WeaponInitBullet 0x1a4bf0:226-245] (the decompile): the head → aim·256 ray snapped by
   `Collide_RayIntersect` (mask 0x20a/0x208), the head↔datum-0 midpoint branch per the record
   bits (fp model, not excluded, not zoomed-scope, +0x68 & 0x10000), `dir = Normalise(aim −
   start)`; the fire-path `dir` construction (the cursor offsets with the one-step aim_offset
   ease) is computed read-only, exact for a trigger tick carrying turn 0 / look 0.
2. **The bullet-line gate** — a `bullet_gate` parameter on `shoot_far_aim`, TRUE at the stand
   sweep call site ONLY (every other call site passes false = byte-identical; run 4's global-law
   lesson holds): fire when the predicted line's crossing of the guard's vertical plane lands
   inside the body box (the feet +0.1..+1.55 span, the lateral 0.3 u); the trigger tick sends
   turn 0 / look 0.
3. **Run 10-c (the first gate form) measured the closed-loop lift UNSTABLE [L]**: the aim snap
   SLIDES along the ground/wall behind the guard as the lift shifts (the [0d:plan] prints: the
   snap z swung −304..−329 for ~0.1-u lift steps), the dy-vs-aim swung −1.27..+1.43, the lift
   slammed ±1.2 (the clamp), NO shot fired, Bond drained dry at the stand. The gate was re-formed
   to the body-box measure with the lift frozen.
4. **Run 10-d: the gate LANDS THE FAR KILLS [L]** — g2 down one round (+57), g0 down ONE TORSO
   HIT at d 17.4 ([audit] SPAWN dir (0.9993,−0.0287,0.0244) → part Some(4,0.113), hp 10 → −22).
   The wall moved to TIME: the per-pass waits cost 5.8 + 21.6 of armed-pack 0x12 drain (death
   mid-pass-2 at 4.0 health).
5. **Run 10-e measured the budget law [L]**: ONE shot per ~110-tick fire-script cycle (the
   aim-bit flap [M 0x1a24c8/0x1a2e38]: the script drops the bit, the raise re-sets it ~20 ticks;
   the 0x1e's script is long), and the +54 shot missed g2's head box by **0.02 u** — the
   [0d:plan]/[audit] pair measured THE SNAP-HIT LAW: the bullet passes THROUGH the snapped aim
   point exactly (the crossing (143.70, 6.38, −301.72) = the aim = the planned line), so the
   runs 6-7 lift −0.22 aimed 0.22 BELOW the head box (+1.60) and the bullet crossed +1.38 —
   0.02 under the box bottom, part None, 31.4 → 0 in 164 ticks. One-shot-per-cycle × 4-5 guards
   × the armed drain = the fight math is UNWINNABLE AS A FIGHT. The ledger item (10)'s premise
   ("the fight math closes at ~10 health margin") is measured FALSE for the ~110-tick cycle the
   0x1e actually runs — the round-11 lever composes the gate with the runs instead.
6. **The yard-south RUN variant (the ledger item (10)'s own fallback) implemented** —
   `NF_0D_YARD_RUN=1` (default 0 = byte-identical; rides NF_0D_STAND_FIRST=1): skip the stand
   sweep, keep the proven zero-cost escape walk (the wire dodge), keep `skip_return_fire` set
   through the yard legs, and gate off the en-route fight acts (the trench grunts #25/#26, the
   yard-east gunner #8 — the fights ARE the drain the run dodges; the objectives — the ST-3
   plate, the photograph, the C5 door — need no guard kill).
7. **Run 10-f: the deepest death the family has measured [L]** — the ST-3 plate AND the display
   trench reached (t23384), then the roof shooter #29 (138.9, 9.0, −325.3, d 18.2 — 6 chips for
   10.8) + the armed yard's 0x12 cadence (#1 16.0 in 9 hits) drained the 31.4 during the
   trench→plate return leg, death at (139.13, 5.08, −307.49).

No guard code changed, no damage/health constants changed, no top-ups. The new machinery: the
additive read-only `planned_shot` (+ the `PlannedShot` struct), the `bullet_gate` parameter
(false at every call site but the stand sweep's, which only executes under NF_0D_STAND_FIRST=1),
and `NF_0D_YARD_RUN=1` (default off). The knobs-unset tree is byte-identical — the whole-binary
runs below are the proof.

## How it was verified

Every lever was judged on its own real run through the real-input path (NF_ISO, fresh load, no
teleports after the load). Runs this round (all [L], the direct test binary, --test-threads=1,
--ignored): run 10-a t22674 (the wrong-config roll — no corridor env; the landing burst), run
10-b t21587 (the ride death without the pit-kill dwell — g18's item (8-iv) values 17.8+17.0),
run 10-c t22674 (the oscillating lift loop), run 10-d (the gate's two certain kills, death
mid-pass-2), run 10-e t22688 (the 0.02-u miss, the snap-hit law, the one-shot-per-cycle law),
run 10-f t23384 (the yard run: the plate + the trench reached, the roof #29 + #1 drained it).

Whole binaries on the final tree (knobs unset):

- `walkreal_0700000d_live`: **3 passed / 0 failed / 1 ignored** (103.7 s).
- `interactables_0700000d_live`: **11 passed / 0 failed** (30.9 s).

The fast runner (`tools/test/suite.py`) was started in the background in the first minutes per
the driver's instruction; this sandbox killed it between tool calls (the rounds 1-9 signature —
`suite.out` stayed empty and no suite process survived the first tool call), so the per-binary
foreground runs above are the primary evidence, as in rounds 1-9.

Crates touched: `crates/nf-view` (scene.rs: the additive read-only predictor; one test file).
The scene.rs change adds a method and a struct — no existing code path moves; the two binaries
that exercise the 0d paths ran whole above. The other levels' walkreal binaries were NOT re-run
this round (the window did not fit them; the additive change touches no existing behaviour —
`planned_shot` has no other callers, grep `grep -rn planned_shot crates/` names only scene.rs
and walkreal_0700000d_live.rs).

## What's left

- The walk itself: still red — the round-11 lever (ignore message item (11)): COMPOSE the two
  proven pieces — the box-centre fire gate (the crossing vs the clear box centre, a ~0.2-u
  sphere; the +0.02-miss evidence says the aim rides the BOX, lift 0 — the runs 6-7 −0.22
  compensated the snap-miss branch the gate now refuses to fire on) to thin the pack with
  CERTAIN one-round kills at the stand (~1 shot per ~110-tick cycle, the drain falls per kill),
  THEN the yard run with the thinned pack; the run legs' remaining drains are #29 (the roof
  above the trench — a targeted kill from the trench, the head act, or a lower-exposure return
  route) and #1 (the yard-east, 16.0 in 9 hits).
- Live checks for the main agent: none new.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy

The three fallbacks in order (09:10-09:13 UTC 2026-11-20):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (the rounds 1-9 signature).
2. python3 urllib: no output within 25 s (the timeout wrapper reaped it with no response line).
3. wget: no output within 30 s (no response line).

All three failed — the same ntfy-unreachable signature as rounds 1-9. The payload that would
have been sent: `retune-nb-0d-10: walk still red - the Y-law gate LANDS the far kills (g0 one
torso hit d 17.4), the snap-hit law measured (the bullet passes through the snapped aim), the
fight budget law measured (one shot per ~110-tick cycle vs the armed drain - unwinnable as a
fight), the yard run reached the trench (t23384, the deepest yet); the round-11 lever composes
the gate + the run; whole binaries walkreal 3/0/1 + interactables 11/0`.
