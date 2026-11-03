# level-0d-real-health-crossfire-4 — report

Branch `s/level-0d-real-health-crossfire-4` (cut from `origin/s/level-0d-real-health-crossfire-3`
9f9f9631; origin/master f3b24398 was already contained via -3's aa8f67d2, so the merge is a
no-op and the branch is -3's tip plus this session's route work).

## Status: blocked (the C5 real-damage walk is not green yet) — the ladder-top phase is won,
## the death moved 4032 → 6159 past it onto the wire, and the blocker is now precisely measured

The driver's repro held exactly: on the -3 tree the walk dies at tick 4482 at the ladder top
(213.07, 17.37, −234.51), 10 hits for 100 (#16 38, #18 32, #28/#27 the roofs), and it
reproduces tick-for-tick on this sandbox. The -3 report's 16287 run does NOT reproduce here:
the same committed test code on this sandbox dies at 4482 (the driver's number), and repeated
runs inside one sandbox are identical (4032 twice, 6159 twice). **The walk's outcome is
sandbox-stable but not sandbox-portable** — every prior report's "final run" numbers are
sandbox-local. Route engineering has to be done against the running sandbox's own ledger.

## What's done (all route-side in walkreal_0700000d_live.rs; NO crates/*/src diff this session)

1. **THE WORLD-RAY GATE in `shoot_far`** — a leg fires only when the shot's own law (the scoped
   bullet pick the flight walks [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058's query])
   sees the eye→chest line CLEAR. The -3's #32 leg fired 2 rounds into a ray its own probe
   printed 0.960-blocked; the west-roof sweep's pass 3 ran at clip 1 spare 25 with a third
   walker alive — the dry sweep is what fed the 4482 death. The gate is safe for the sweep: the
   landed passes' rays print CLEAR (the -2's "kills through blocked rays" were against the
   SETTLED-spot probes; the mid-walk crossings this sweep kills on measure CLEAR).
2. **THE STRIP-MOUTH #32 KILL — landed for the first time ever (+528)**, then moved to the bay
   phase. The kill mechanics that landed it: the box widened to the whole patrol band INCLUDING
   the east end (163.3, −214) — the -3's two probe rays died at 0.960/0.965 on the ~1.7-high
   ground obstacle at (160, −217) with targets at x 158.5/158.6 (the band's WEST spots, the
   obstacle between stand and target), while a ray to the EAST end ends BEFORE the obstacle's x;
   the gate holds fire through the west spots and takes the crossing. The kill then moved: the
   measured trade is that the 528-tick dwell lets the wake's reposition walkers settle into
   x ~151.4 and shoot the CLIMB (the runs with the landed kill: 86 on the ladder face, death
   4032; the failed-400 leg: 16; the -2's no-dwell climb: silent). **The climb-timing law: a
   short strip-mouth dwell = a silent climb.** The bay-phase leg (after the annex drop, the same
   gate) is in the route but untested this session — the death precedes it.
3. **The ladder-top order: #16 first (+209, every run), then the sweep with 900-cap passes.**
   The 900 caps (not the -3's shorter experiments) buy the patrol-crossing catches — the sweep
   killed #28 at +194 (caught in the north sub-band −238..−241 where the slab-edge ray clears).
   With #16 dead first the sweep's exposure is #18's bay fire only.
4. **The climb-timing law applied**: with only the #10 kill (+227) at the strip mouth the climb
   is SILENT (zero hits — the -2's reality restored), #16 dies at +209, and the route reaches
   the wire phase alive.

## Where it stands (the -4 run 7 ledger, the committed tree, deterministic)

Death at 6159 at (207.87, 16.14, −244.93) — ON THE WIRE, past the ladder-top phase. The pool:
100 = the climb's tail 46 (#28 ×2 — one with a picked-up weapon 0x0 — and #27, from ~1300 ticks
after the wake) + the sweep phase 48 (the two settled survivors #27/#29 firing while their
passes wait) + the wire's first hit 6 (#18). The blocker, measured: **#27 and #29 never enter
the clear-ray sub-band** — their settled spots (−245.6 / −252.8) sit behind the roof slab edge
(the ladder-top eye is 4.9 BELOW the roof surface over ~64 u, the -2's 0.972-0.974 law), and
their patrols keep them south of −243 where that holds. #28 entered −238..−241 and died there.
The two survivors then trade the wire ride at d 57 (the -2's one-hunter ride cost 54; two
hunters cost ~96 — the ride is unwinnable with both alive).

The next session's levers, in order of promise:
1. **A higher eye for the settled walkers.** The ladder-top eye is 4.9 below their roof
   surface; the -2's scan blessed the "wire-mid" stand (CLEAR at d 52 to #28's settled spot —
   not standable, it is the ride itself). The route's own roof platform (19.65, x 153-158,
   z −268) and the catwalk are unprobed stands: the [0d:ray] audit in a probe run from those
   spots settles whether the slab edge clears from a different azimuth.
2. **The annex-phase #32 leg** (already routed, the gate measures the bay stand live) — its
   0.7-per-hit trickle (the -3's 86.4 across 124 hits) only matters once the ride is clean.
3. The ride's exposure: the strafe-ride already rides the MOVING quad law (the retained
   walk-ticks zeroing [M 0x193600:129-134]); a still-attached survivor fires ~16/200 ticks —
   the ride itself is ~96 with two hunters, ~54 with one, ~0 with none.
4. The wire-phase #18 hit (6): the bay leg is right after the annex drop — one phase later.

## How it was verified

- `walkreal_0700000d_live walk_arrival_to_the_c5_door` (the master-green twin): **PASSED** on
  the final tree — the full mission walked + completed in 24926 ticks, 0 stuck, 0 falls. The
  route changes keep the C5 walk's route alive.
- `walkreal_0700000d_live walk_arrival_to_the_c5_door_real_damage`: run 7 times this session
  (the -3 tree repro 4482 twice; the fix iterations 4032 ×3 → 6159 ×2, deterministic per tree).
  Still red — the death and its ledger are quoted above.
- `walkreal_0700000d_live` whole binary: 1 passed / 1 failed (the two tests it has).
- **No `crates/*/src/` file changed this session** (git: the branch touches only
  `crates/nf-view/tests/walkreal_0700000d_live.rs`, WORKLOG, this report), so the other
  walkreal binaries and the rest of the suite are byte-identical to the -3/master state by
  construction. The every-`walkreal_*_live` sweep and the full-suite total line did not fit
  the sandbox window (~70 min of it went to the repro iterations); they are owed before merge,
  and the cross-sandbox nondeterminism finding argues for running them ON THE MERGING
  SANDBOX, not trusting any worker sandbox's numbers.

## Live checks for the main agent

1. The -3's checks stand (its #32 spawn/post question, the (160,−217) ground-obstacle [D]
   coll-scan, the -2's checks 1-3).
2. NEW — the west-roof slab-edge azimuths: from the roof platform (≈(156, 20.5, −268)) and the
   catwalk, is the eye→chest ray to a SETTLED walker (151.4, 22.7, −245.6 / −252.8) clear?
   `fieldprobe`/gdbwatch on the walker's health byte from those stands — expected if the
   azimuth clears: the same 1-2-round kills the ladder top lands on the north sub-band.
3. NEW — the cross-sandbox determinism: the -3's 16287 death does not reproduce on a fresh
   sandbox (4482, then the -4's 4032/6159 with the route changes). If the main agent's PCSX2
   pass can run the SAME branch twice on two machines, the divergence point would pin whether
   a std-HashMap RandomState or an audio-driver real-time read leaks into the sim (the port's
   determinism rules forbid both; nothing else in the tree is machine-dependent).

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
