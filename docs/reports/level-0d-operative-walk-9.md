# level-0d-operative-walk-9

Branch `s/level-0d-operative-walk-9` (origin/master 385ebb66d + `s/level-0d-operative-walk-8`
merged, clean). Task: the OPERATIVE-WALK-8 ledger's lever (1) — the pair's CROSSING SWEEP with
the box act run BEFORE the retarget loop on
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`; real input only, no
top-ups, never lower damage or health; the plain replay and the d2 twin stay byte-identical.

## Status: still red — the crossing catch is REAL, the death did not move

The walk still dies in the ladder-top phase. The lever's own real run: tick 4161 at
(213.07, 17.37, −234.51), 10 hits for 100.0, 0 return-fire fights (the walk-8 final-tree roll:
4160 — one tick apart, the dice family unchanged).

## Lever (1) — the pair's crossing sweep, WIRED AND MEASURED

- The wiring (the -3 kills' own law verbatim): the walker-catching sweep hoisted BEFORE the
  Operative retarget loop — `kill_guard_at_present` at the walkers' band box
  (151.2, −246.8) ± (6, 11), cap 900, the nearest-live pick, gated `real_damage` at
  difficulty 1 (the plain replay and the d2 twin keep their trees byte-identical).
- **THE CATCH LAW IS REAL**: the first pass caught **g29 at +200** — she was 0.972-blocked at
  the pick census (151.37, 22.70, −245.55, the run-4380 mid-band law) and crossed clear
  inside the pass; the -3 family's +194/+222/+236 shape, measured on this tree.
- **THE DEATH STAYED 4161**: the trio's first 64.0 landed BEFORE the sweep could open
  (~3800): #29's 32.0 at ticks 3042/3222 and #27's first hits at 3398/3578 — the sweep sits
  behind the #16-first (+209) and bay-duo (+195) laws — so the +200 catch removed only
  FUTURE fire. The remaining shooters delivered the 100 inside the same window: #27 32.0 in
  4 hits, #29 32.0 in 2, #10 22.0 in 2, #28 14.0 in 1, #18 0.0 in 1.

## Lever (2) — the clear-box pick refinement, MEASURED A NO-OP

The data-driven refinement: each pass rides the retarget pick's torso-first clear-box law
first (`pick_trio_target` + `shoot_far_box_fast`, cap 450, the zero-damage blacklist into the
SHARED `stalled_op`), and only when NO clear box exists anywhere in the band falls back to
the nearest-live chest crossing act (cap 900). Its own real run: `pick_trio_target` returned
**NONE** at the sweep moment on this roll — no clear torso box anywhere in the band; #27
stood at her SOUTH spawn (151.35, 22.70, −255.80), whose clear-ray law is the -4/-5 walks'
wake dump, not this roll's moment. The fallback ran the identical measured behavior — the
dice tick-for-tick identical, the death 4161. The refinement STAYS (strictly-better-or-equal
on any dice: the pick act fires only on a verified clear box, else the measured fallback).

## The structural floor, confirmed

The trio's first 64.0 lands in the 3042–3800 window that the #16-first and bay-duo laws
occupy ahead of any sweep; the sweep catches only future fire; the act-fire dice correlation
(the walk-6 law) held — 10 hits for 100.0 on every roll this session.

## The refreshed ledger (in the ignore message)

NEXT LEVERS: (1) the sweep hoisted to the FIRST ladder-top act (before the #16 deck act) —
the trio is present-but-pre-cadence there (the first hit lands 3042), the reposition-era
crossings are the -3 law's most catchable window (the placements 145.6–145.8 are CLEAR rays
from the top), and #16's zero-delivery law says her +209 kill can absorb the sweep's
~400-tick delay; (2) the port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] (its
own task); (3) the yard levers.

## Verification

- `walkreal_0700000d_live` whole binary (run per-test — the sandbox OOM-kills the
  3-sim parallel run): the plain top-up replay **passed** (28.5 s, byte-identical — the
  `real_damage` gate kept it out of the new legs), the d2 real-damage twin **passed**
  (42.0 s), the Operative real-damage walk **ignored** (dies at tick 4161, measured on its
  own runs: the nearest-live shape 4161, the refined pick 4161 tick-for-tick identical).
- `interactables_0700000d_live`: **11 passed, 0 failed**.
- The full suite did NOT fit: this sandbox reset at session start (clone, both downloads,
  the toolchain and the first build all foreground — the background fast-runner is killed
  between tool calls here, as in walk-8), and the remaining window went to the lever runs
  and the two mandated binaries. What ran whole: both binaries above, three times for the
  lever walk. What's left for the next session: `python3 tools/test/suite.py` on this tree.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).
