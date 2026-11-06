# level-0d-real-health-crossfire-5 — the 0d real-health route (low priority)

Branch from origin/s/level-0d-real-health-crossfire-4 (merge origin/master first) and read its report ("Where it stands"). The level is already completable on master (the non-damage twin is green); this is the real-health survival walk only.

On this route walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage (green on master) dies on the wire at tick 6159: roof guards #27/#29 settle behind the roof slab edge where the ladder-top eye cannot reach them, and two hunters cost ~96 on the ride. Report lever 1: probe higher stands (the roof platform at 19.65, x 153-158, z -268; the catwalk) for a clear ray to their settled spots; then the annex #32 leg.

Hard rule: every test that is green on master in walkreal_0700000d_live must stay green. If the new route cannot keep the C5 walk alive, keep the C5 walk on master's route. List each test's result. Real input only; no top-ups.

---
claimed-by: sandbox3 at 2026-11-06T04:40Z

---
## Report (done, 2026-11-06T05:07Z)


---
## Report from the worker (done, 2026-11-06T05:07Z)

# level-0d-real-health-crossfire-5 — report

Branch `s/level-0d-real-health-crossfire-5` (cut from `origin/s/level-0d-real-health-crossfire-4`
c2c4a6efd, origin/master merged at 42b683045). The tree touches ONLY
`crates/nf-view/tests/walkreal_0700000d_live.rs` and the new
`crates/nf-view/tests/probe_0d_roof_rays.rs` (#[ignore]d probe) — **no `crates/*/src/` change**,
so every other test binary is byte-identical to the merged master/-4 state by construction.

## Status: DONE — the C5 real-health walk is green

`walkreal_0700000d_live`, whole binary, final tree: **walk_arrival_to_the_c5_door PASSED (the
master-green twin), walk_arrival_to_the_c5_door_real_damage PASSED**,
walk_arrival_to_the_c5_door_real_damage_agent ignored (pre-existing). The real-damage run
completed the FULL mission: **25275 ticks, stuck 0, falls 0, end health 22.2, lowest 22.2 —
62 hits for 64.2 + 13.6 fall damage** (the two route falls, the annex drop and the beam drop).
Real input end to end; no top-ups; the only pos write is the documented boarding stand-in that
predates this task.

## The lever that won: the slab-edge law is HEIGHT-ASYMMETRIC per aim line

The -4 blocker: the west-roof guards #27/#29 settle behind a rooftop obstruction and the ride
with two hunters costs ~96 — unwinnable. This session's probe (`probe_0d_roof_rays`, run with
--ignored --nocapture) measured the actual law from the route's own stands:

- The chest line (pos +0.3) dies at 0.80-0.974 on a rooftop obstruction around z −249..−252 —
  but the HEAD line (pos +0.9) clears it from the SAME stands (the platform-kill stand, the
  dock edge, the drop point, the strip, the ladder top), and from the LADDER TOP the CHEST ray
  to the south spawn (151.06, −255.81) and to g28's walked spot (151.43, −241.25) is CLEAR —
  the -2 "0.972-0.974 law" was the mid-band obstruction, not the whole roof.
- The wake spawns the west roof DIRECTLY at stable combat spots (the probe's wake dump: g27
  south (151.06, 22.70, −255.81), g28 (146.4, −237.7) in the parked timeline but (151.43,
  −241.25) in the walked one, g29 later at (151.05, −250.82)) — there is no reposition window
  to catch, so the sweep's crossing-catch design was chasing a phantom.
- One head hit drops a guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head, damage.rs
  part 5].

## The route changes (all log-only legs; the twin stayed green on every iteration)

1. **The ladder-top west-roof kill order**: g28 FIRST (chest act, her walked spot (151.4,
   −241.3) — down at +224), then g27 (chest act, her south spawn — down at +522), then #16
   (+193), then the g29 south-walk chest leg (she walks −250.8 → −255.7 out of the obstruction
   band; the world-ray gate holds fire until her line clears — the sweep took her at +337 this
   run). **All three roof guards die before the wire grab — the ride carries ZERO hunters** (the
   -4 ledger's ~96 ride cost is gone; #18's one 14 is the only wire-phase hit left).
2. **The yard garrison sweep at the beam drop**: the yard grunts (#6 the C5 door corner d 7,
   #3 the plate path, #0, #5, the trench pair #25/#26) sleep until approached [L census:
   enabled false] and their 0.6-1.2-per-hit trickle accumulated 86.4 across 98 hits in run 4 —
   death by accumulation at 16441. The drop point's flat-yard rays are clear, so #6 (+196) and
   #3 (+217) die ASLEEP before the south walk, and #25/#26 die asleep at the display trench
   before the camera switch.
3. **shoot_far_aim gained a BOX mode** (`shoot_far_box` via `clear_box_aim`): the aim point is a
   REAL clear hit-box centre each tick (the part-5 head first, ray-tested exactly like
   damage_log::exposed_impl). Run 2 measured why: the fixed +0.9 head guess fired clear-ray
   rounds that passed under the head box — a box point the port's own law returns is on a box
   the bullet reaches. The chest path (`shoot_far`, aim_dy 0.3) is untouched.

## Measured dead ends (so the next session does not re-walk them)

- **A 600-tick aimed duel at strip 2 is NOT safe** (run 2): the strip's zero-hit law holds for
  the baseline's short stands only — the bay guards' spread fire lands through the low opening
  the single-line probe calls blocked (#18 3×16, #10 2×16), and g27's counter-fire cleared the
  obstruction (16 at d 66). The strip phase is back to the baseline legs exactly.
- The driver's lever-1 stands measured ([0d:ray] table in the probe log): the roof platform w
  (155, 20.35, −268) — ALL spawn rays blocked 0.206-0.578 (its own south structures); the roof
  platform nw (158, −268.7) — g29's spawn HEAD clear only; the catwalk mid/e — g27 clear, g28
  blocked, g29 blocked. All are POST-ride stands, so none can save the ride; the ladder-top
  chest/head lines above are what saved it.
- The bay #18 in-room kill leg failed again (400 ticks, both runs) — her 14 (one wire hit at
  d 74.5) stayed inside the budget; the leg stays as-is.

## How it was verified

- Six full runs of the 0d binary this session, deterministic per tree: baseline death 6855 (the
  -4 tree, 10 hits for 100); run 2 (the strip-duel attempt) death 3283 — reverted; run 3 (the
  ladder-top order, g28 box wrong) reached the beam drop at 14640 with g28's 62 on the ride;
  run 4 (g28 first) reached the yard at 16441; run 5 = run 6 (the final tree): **BOTH tests
  green**, the ledger quoted above. The twin passed on EVERY iteration (5/5).
- Each binary whole with its pass count: `walkreal_0700000d_live` — 2 passed, 0 failed, 1
  ignored (the Agent sister; its levers were this task's set and it now builds on a green
  Operative base). No other binary is affected by construction (no `crates/*/src/` diff; the
  probe file is #[ignore]d). The every-`walkreal_*_live` sweep and the full-suite total line
  did not fit the sandbox window (~85 min; five of them went to the measured iterations) — they
  are owed before merge, and the -4's cross-sandbox finding argues for running them on the
  merging sandbox.
- `cargo test -p nf-view --test walkreal_0700000d_live --no-run` clean; no new compiler errors
  (the pre-existing warnings unchanged).

## What's left

- The full suite + the every-`walkreal_*_live` sweep before merge (owed, see above).
- The Agent sister (`..._real_damage_agent`, ignored) — the d2 snipers #21/#36 sit outside the
  fights' window; the lever set this task built (the ladder-top west-roof kills + the yard
  asleep-sweep) is now on the table for it, but its own route work is untouched.
- The bay #18 in-room kill leg keeps failing at the 400 cap (both runs) — worth a probe of the
  in-room ray law if the next session wants her 14 off the wire phase.

## Live checks for the main agent

None required this session — no visible change (no `crates/*/src/` diff, no render surface
touched), and the walk's own asserts are the verification.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
