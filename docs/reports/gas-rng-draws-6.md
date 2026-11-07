# gas-rng-draws-6 — report (2026-11-05)

**PARTIAL, the task's named reds landed** — both walks report-5 left red are now green whole
(03: 6 passed / 0 failed / 2 ignored; 14: 9 passed / 0 failed / 1 ignored), master is merged
with the gas draws carried through, and mp_perf_16's digest is re-pinned on the merged tree.
1b regressed by the merge (Drake's fight) and carries with a fresh ledger; the full suite did
not fit the window. Branch `s/gas-rng-draws-6` (cut from `origin/s/gas-rng-draws-5`
e4e50af06; origin/master merged at cd64fbbc0).

## Done this session (each verified on this tree)

- **The master merge, with the gas RNG landing carried through** — three conflicts resolved
  and every master-side `Gas_Init` call re-based to the real draws:
  1. `gas.rs`: `init_ex`/`init_ex_rot` keep the rng signature; master's kinds-9/10 hidden
     cloud emitters (fx-gas-and-blasts row 5) ride inside the rng body (the `9 | 10 => [0.0; 3]`
     velocity arm, the kind-10 life half, the ChildSeed record). The kind-9/10 child spawn
     (`push_child`) keeps master's counter-hash stand-in — documented, and no game caller in
     the port's scope creates kind-9/10 parents, so no stream moves today.
  2. Master's newly-merged sites are REAL `Gas_Init` calls and now consume the game's draws:
     the laser row-11 second puff (0x1cf1ec's `li v0,0x15` life), the rigid-body buoyancy row-7
     puff (RB_Update 0x1d6eb4), the flicker spark (0x1abc3c, its `Rand_Rand(5)` mesh draw was
     already the stream's), and the water-ripple callers of `Env_WaterRipple` (0x17c118, whose
     port fn already took rng). The non-Gas_Init stand-ins (the swoosh ribbon, the GT burst)
     stay on `init_hash` — the game consumes no draws there.
  3. `mp_perf_16_live`'s pinned digest re-measured on the merged tree and re-pinned:
     **0x986f_90e8_e5d6_b12f, kills 35** (both parents' game-faithful changes ride one
     stream: the gas draws + master's registered-law pickups, the dead-bot row resets and the
     bot pain/death barks). Binary green 2 passed / 0 failed.
- **03 (the full route): the d2 real-damage run GREEN, whole binary 6 passed / 0 failed /
  2 ignored** (363.69 s; was 5/1+1i). Report-5's named first step, done: a cheap `#[ignore]`d
  probe test (kept in the file, run with `--ignored --nocapture`) dumped the door-8 cluster's
  LOAD state — **#18 (27.92, 6.25, −5.57) hp 12 weapon 0x1c, #2 (30.69, 6.25, −7.55) hp 12
  weapon 0xe, #19 (37.01, 6.25, −15.04) hp 12 weapon 0xe, all disabled state 0x1, and door #8
  (35.62, 6.51, 2.71) closed state 0x2 pos01 0.50** [L]. The staged mouth-clear's anchor was
  the POST-open pocket (38.7, 0.2) r 6.0 they walk to AFTER the door opens — it never matched
  the load spots, so the block never fired. Re-anchored to the load band (8.0 around the
  centroid (31.9, 6.25, −9.4) — the probe shows no other hostile in it), plus a bounded
  hinge-side settle loop (5 × wait(90) + clear_near(9)) so the popped members are fought
  through the doorway the way the green runs fought from (36.2, 5.7). The failing run now
  walks to 93135 ticks and finishes at 36.3 health; the green streams see only corpses at the
  gate and stay byte-identical (all six tests re-run green).
- **14 (the infiltration): whole binary 9 passed / 0 failed / 1 ignored** (149.59 s; was
  4/5+1i). Three fixes, all in the test file (no `crates/*/src/`, no `tests/common/` changes):
  1. The corridor-door jamb wedge (four tests): the merged stream lands the "corridor west of
     the door wall" step 0.97 from its band, at (30.33, 0.55, 35.44), and the direct south
     steer hits the door wall's west jamb 0.17 along — with door #834 fully open (state 0x4,
     pos01 1.00) and "drones nearby: none" [L the four identical 6000-tick stuck dumps], so
     the block is the wall itself. A bounded raw-tick west nudge (slide to x < 29.9 along the
     jamb's north face, the lane the probe walked at x ≈ 29.7-30.0) now runs before the south
     leg at both chain copies; it is a no-op when the step already arrived, so green streams
     stay byte-identical.
  2. The real_damage descent assert: the walk itself arrives in-band on the merged tree
     ((29.79, 0.56, 37.59)) — the failure was the landing pin being sampled AFTER
     `clear_hostiles`, whose 10-fight running clear had dragged Bond to (25.97, 0.65, 36.40)
     [L]. The pin now samples the walk's arrival before the clear; nothing was weakened.
  3. The delta exit: E is edge-triggered, so the old held-use loop spent its single edge on
     its first tick — and the d1 run's pocket held live enabled guards (#42 (25.6, −23.7)
     hp 3.0, #47 (27.5, −20.9) hp 7.4, #49 (28.9, −18.9) hp 10 [L the diagnostic dump]) whose
     bodies eat the mask-4 use ray. The exit now pulses E every 6 ticks while squared up on
     #944, after a bounded 12-u pocket clear; all three difficulty sisters load 07000015 warm
     and pass (the load, ch 0xCE, the obj-5 fold and the arrival pins all hold).
- **The re-based-sites sweep (rule 10: `crates/*/src/` changed)** — each binary whole,
  `--test-threads=1`, slim ISO:
  - `mp_perf_16_live` **2/0** (the re-pin above);
  - `walkreal_07000002_live` **3/0 + 1i** (70.64 s) — untouched, still green;
  - `walkreal_07000016_live` **14/0 + 2i** (70.65 s) — the draws-5 laws hold on the merged tree;
  - `goggles_render_live` **8/0** (9.65 s) — the gas-draws neighbour;
  - `walkreal_07000003_live` **6/0 + 2i**, `walkreal_07000014_live` **9/0 + 1i** (above);
  - `walkreal_07000001_live` **5/3** (144.63 s) — the SAME three reds draws-5 reported
    (the exit-fade test, the plain contact walk, the d3 00agent run; d1+d2 real-damage green).
    No regression from the merge; report-5's 01 ledger and next steps stand.
  - `walkreal_0700001b_live` **15 passed / 2 failed / 2 ignored** (145.72 s; was 17/0+2i) —
    the one merge regression, below.

## What's left (with the fresh ledger)

1. **1b: Drake's fight (both `walk_arrival_to_drakes_defeat` and its real-health sister).**
   Bond survives; Drake does not die in the fight window — the assert now prints it: **Drake
   hp 60.0 left after 9000 fight ticks** from the hold spot (32.0, −1.7, 0.0) [L this tree].
   The window is NOT the lever (60 of 100 hp untouched after 9000 ticks — Bond's shots from
   the hold spot do not land on the merged stream; Drake's float path or the aim line moved
   with the merge, whose nis.rs/space_missile.rs/footstep changes shifted this level's
   stream). Next session: one probe run sampling Drake's pos + Bond's aim hit every 60 ticks
   of the fight, then re-derive the hold spot the way 03's mouth-clear was re-anchored. The
   fight cap is left at 9000 with the health in the message — an honest wider window.
2. **The full suite** did not fit: chunk_suite.py's run phase is ~3 h on this box (the
   standing measurement) and the window was ~2 h. Every binary named by rule 10 for the files
   I touched ran whole and is listed above; the untouched-level binaries (04, 05, 06, 07, 08,
   09, 0a, 12, mp others) were not run — my changes touch no `tests/common/` file and the src
   changes are the gas sites (laser impacts, buoyancy puffs, flicker sparks, ripples), whose
   per-level consumers are the walks listed.
3. **The 00-agent 03 walk** stays `#[ignore]`d (draws-5's ledger stands; the door-8 work here
   is directly its starting material).

## Live checks for the main agent

1. 1b's Drake float: on the merged stream, WHERE does Drake float during the fight from the
   hold spot (32.0, −1.7, 0.0)? gdbwatch/fieldprobe his obj position every second from the
   fight's start — the port's shots from the spot stopped landing (60 of 100 hp untouched in
   9000 ticks), so the spot vs his new path is the question.
2. (carry) Report-5's checks 1-3 stand (01's frozen slide leaf, 03's door-8 pre-door spots —
   now answered by the port-side probe, 16's pit wire).

## Verification notes

- No decompile-backed test was weakened: the 03 mouth-clear gate got a true anchor and a
  settle loop; the 14 descent pin kept its band and moved BEFORE the running-fight clear (the
  pin's own meaning — the walk's arrival); the delta exit's load assert is unchanged.
- The stream discipline: the green streams' byte-identity at the touched seams is by
  construction (self-gating on corpses / already-in-band positions) and by re-runs (all six
  03 tests, all nine 14 tests, 02/16/goggles re-run whole on this tree).
- The token never left the clone URL; `git status --short` was checked before every commit
  (only the named test/doc files were staged).

## Usage

not visible to me (no token, credit or context-window figures are exposed to this session;
nothing in the sandbox reports them).
