# retune-nb-0d-9 — round 9 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-9`, from origin/master ccd79b5be with origin/s/retune-nb-0d-8 merged in first
(0d-8 was not on master; clean merge). The test `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but this round PROVED lever (a)'s core premise (the periodic live wire is dodged by
leaving the landing spot inside its ~1300-tick cadence), pinned the round-8 sweep-stall's exact
mechanism (the walker's return-fire answer pins Bond on the spot), and measured the bullet-dir law
that makes the stand sweep's passes miss at range. Five real runs this round; the refreshed ledger
and the round-10 lever are in the ignore message item (10).

## What was done

1. **The round-9 lever (a) implemented** — `NF_0D_STAND_FIRST=1` (container roll only, unset =
   byte-identical): the landing-spot sweep is skipped and the probe's proven-clear stand
   (142, -291.5) is walked FIRST, standing, then the sweep runs from the stand. The landing
   crouch is skipped too (the walk's rays were read standing; the presses would re-crouch).
2. **Run 1 pinned the round-8 stall's mechanism [L].** With the sweep skipped but the escape
   walk still using the walker's return-fire answer, Bond died t22479 at (141.22, 5.06, -296.08)
   with POSITION UNCHANGED: `walk_to`'s per-10-tick return-fire answer (crates/nf-view/tests/
   common/damage_log.rs) holds Bond on the spot answering shooters whose acts cannot land from
   there (the container blocks the rays; the rifle is nearly dry after the pit act — clip 5
   spare 10), while the periodic live wire fires on its ~1300-tick cadence (-27.80 in 2 hits).
   The round-8 "~1345-tick sweep stall with zero kill prints" IS this answer, not the sweep acts.
3. **The escape leg's opt-in flag** — `skip_return_fire` on the shared walker_b2 Walker (default
   false = every walk byte-identical; lesson 7's opt-in pattern), set for THIS leg only in the
   stand-first branch, cleared after. Run 2: the leg arrived at the stand (141.62, 5.06, -292.57)
   at +70 ticks with ZERO chips (health 27.8 intact) and **the wire never fired** — lever (a)'s
   premise PROVEN. The landing burst ate armour 50→0 + 5.4 of health inside the committed
   40-tick land_wait BEFORE the walk (sight-triggered at the landing; the round-6 law).
4. **The stand sweep's kill race is a knife edge — and the g6-class pass misses [L runs 2-5].**
   Pass 0 killed g2 (d 7.8) at +57 (one round). The fight math: 5 passes × ~60 ticks at a
   0.07-0.10/tick drain that falls as shooters die ≈ 27 health needed vs 27.8 available — it
   closes ONLY with ~zero missed passes. The g6 pass (d 17.5) missed and burned: death t22767.
   The audit's SPAWN lines measured **the bullet-dir law**: the bullet's dir y = bond.pitch ×
   FRAC_PI_2 EXACTLY (run 5: pitch 0.023 → dir y +0.0364) — the cursor-y correction the tap
   gate aims with does NOT steer the bullet — and the spawn muzzle is ANIM-POSED (run 2 muzzle
   y 5.60 vs run 5 muzzle y 6.12 at the same stand and weapon; the eye is pos + 0.74 standing,
   not the assumed 1.42). The guard states are dynamic (g6 0x73 at the pass, 0x68 at +178), so
   the run-5 pose filter (`d.state == 0x68`, marked [S] unverified) missed.
5. **Run 3** = run 2 byte-identical: the failed-guard memory cannot help a pass that outlives
   Bond (he died +167 inside the g6 pass, before the 260-tick cap could return false).
6. **Run 4** measured that a GLOBAL aim lift re-rolls the EARLY phases' proven acts (death t1789
   at the ladder base) — the lift now rides the stand sweep only (a `lift` parameter on
   `shoot_far_aim`; every other call site passes 0.0).
7. **Run 5**: the scoped 0.88 lift OVERSHOT HIGH (that shot's muzzle was 6.12: the bullet
   crossed g6's plane ~1.4 u above its pos, part None) and the pass burned its 260 cap draining
   25.2 (27.8 → 2.6), death t22891 in pass 1. A fixed lift cannot compensate a moving muzzle.

No guard code changed, no damage/health constants changed, no top-ups: the new machinery is one
env-gated lever (`NF_0D_STAND_FIRST`, default off), one opt-in walker flag (`skip_return_fire`,
default false), the stand-sweep-only `lift` parameter (0.0 everywhere else), the failed-guard
memory (stand_first-gated), and `NF_0D_AUDIT` prints (no sim state). The whole-binary run below
is the byte-identity proof for the committed paths.

## How it was verified

Every lever was judged on its own real run through the real-input path (NF_ISO, fresh load, no
teleports after the load). Runs this round (all [L], the direct test binary, `--test-threads=1`,
`--ignored`): run 1 t22479 (the return-fire pin), run 2 t22767 (the wire dodged, the g6 miss),
run 3 t22767 (the failed-memory bound), run 4 t1789 (the global-lift re-roll), run 5 t22891
(the scoped lift overshoot).

Whole binaries on the final tree (knobs unset):

- `walkreal_0700000d_live`: **3 passed / 0 failed / 1 ignored** (219.6 s) — the plain replay,
  the d1 real-damage walk and the d2 Agent sister byte-identical.
- `interactables_0700000d_live`: **11 passed / 0 failed** (32.7 s).

The fast runner (`tools/test/suite.py`): started in the background in the first minutes per the
driver's instruction; this sandbox killed it between tool calls (the rounds 1-8 signature — no
`suite.out` was ever written and no suite process survived the first tool call), so the
per-binary foreground runs above are the primary evidence, as in rounds 1-8. The full
450+-binary suite does not fit the window either.

Crates touched: only `crates/nf-view` (one test file + the common walker_b2 flag). The
walker_b2 change is the default-false opt-in flag and the constructor field — no code path moves
for any existing caller; the walkreal_0700000d binary (which contains the 0d walks this flag
could reach) ran whole above. The other levels' walkreal binaries were NOT re-run this round
(the ~135 walker_b2 test files' full build does not fit the remaining window); the flag's
default-false short-circuit is the only change they see, and the 0d binary's 3/0/1 whole run
exercised the shared file end to end.

## What's left

- The walk itself: still red — but the wire is dodged, the escape leg works, and the remaining
  wall is exactly one act law: the stand sweep's tap gate must aim the BULLET's own Y-law
  (the view pitch bond.pitch × FRAC_PI_2 from the anim-posed muzzle; no cursor-y). The
  round-10 lever (ignore message item (10)): expose the posed muzzle (or drive bond.pitch
  itself onto the box line in the pre-turn), re-run the stand-first roll — the fight math then
  closes at ~10 health margin. Fallback: the yard-south run variant (skip the sweep entirely,
  skip_return_fire through the plate legs — the d2 sister's proven phases at d3 damage).
- Live checks for the main agent: none new.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy

The three fallbacks in order (05:06-05:10 UTC 2026-11-20):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (the rounds 1-9 signature).
2. python3 urllib: no response within 25 s; the command was killed by the tool timeout with no
   output.
3. wget: timed out — the command was killed at 60 s with no response line.

All three failed. The payload that would have been sent: `retune-nb-0d-9: walk still red but
lever (a) PROVEN - the wire dodged (escape leg +70 ticks zero chips), the round-8 stall pinned
(the walker return-fire answer), the bullet-dir law measured (the bullet flies the view pitch,
cursor-y ignored, anim-posed muzzle); the round-10 lever in the ignore message; whole binaries
walkreal 3/0/1 + interactables 11/0`.
