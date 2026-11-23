# retune-nb-0d-15 — round 15 of the 0d arrival-to-C5-door 00 Agent walk: THE WALK IS GREEN

Branch `s/retune-nb-0d-15`, from origin/master with origin/s/retune-nb-0d-14 merged in first
(0d-14 was not on master; clean merge). **The walk
`walk_arrival_to_the_c5_door_real_damage_00agent` is GREEN and its ignore is REMOVED** — the
committed test passes with no env vars, the full mission completing in 32562 ticks, stuck 0,
falls 0. Eleven real runs this round (15-a..15-k plus the two committed-default verification
runs); the mechanism that broke the 14-round wall was Bond's CROUCH STATE at the return.

## What was done

1. **The round-14 lever (a) — the return-leg reshape — measured NEGATIVE on its own roll**
   (run 15-a, `NF_0D_RETURN_FAST=1`): the direct trench→stand fast leg re-rolled the crossing
   worse (death t23386 vs the pinned t24195, the same 24 hits 80.8; #6's 3-hit burst at d 34.8
   landed mid-crossing). With the thinning added it also died (15-c t23807, 15-d t23831).
2. **The round-14 lever (b) — the #6 north act from the yard-south-1 stand — PROVEN**
   (`NF_0D_NORTH_S1=1`): the certain box act on the #6 corner (137.1, −275.5) from the
   yard-south-1 arrival (138.09, 5.02, −291.04), d ~15.6 across the OPEN yard — **down at
   +81 ticks on every roll that fired it** (15-b/15-c/15-d/15-e) — the second confirmation of
   the round-14 open-line convergence law (the 13-c refusal was the door-corner cover from
   the wire-dodge stand, not the distance). Its roll reached t24481 — the furthest ever.
3. **The stuck dwell identified**: with #6 dead, the return's "back to the plate" fast leg
   burned its full 1200-tick cap stuck at (136.50, 4.78, −308.15) (15-b/15-e identical deaths)
   — the waypoint (138.0, −305.0) r 1.5 sits behind the plate prop from the south approach.
   The east-routed fix (15-g, `NF_0D_RETURN_EAST=1`, the waypoint (141.0, −303.0)) ALSO stuck
   at (139.15, 5.01, −311.65) — and even the committed `walk_to` nav web stuck at the same
   band (15-h: 6000-tick cap).
4. **THE MECHANISM: Bond entered the return CROUCHED.** The 15-h stuck diagnostic read
   `crouched=true head_bump=true` — the STAND_CROUCH press chain left an odd toggle count and
   a crouched Bond's collision profile head-bumps the plate-zone prop; standing, the d1/d2
   sisters cross the same legs green. The guaranteed stand press (the 5-tick toggle idiom,
   conditional on the actual state) + the nav web **crossed the trench edge at +355 ticks —
   the first northward crossing ever on this walk** (15-i).
5. **The #4 mover killed at the stand**: the second leg runs fast (the yard floor is
   fast-leg-safe north of the edge), and the `NF_0D_NW_KILL=1` stand act killed #4 at **+46
   ticks** (d ~5.5, the close-range convergence; all five measured park spots sit inside its
   tol-5 box: (128.5..133.3, −281.3..−282.5) across 14-e/15-a/15-b/15-c/15-d). The other #4
   acts measured: the certain plate act's box-centre line blocked (0.139 / 0.064 from two
   angles — 15-c/15-k), the plate head act found the park band empty (the park completes after
   the plate arrival; the free act left the death byte-identical — the third free-act
   confirmation), the early head act at the open stand drew the converged pack's joint burst
   (#1 the yard-east armed (164.2, −286.7) + #0 + #4, 6 chips in 15 ticks, death t22911
   mid-act — the 14-i signature: the acts' aim phases ARE the exposure).
6. **The green composition committed**: run 15-k (the flagship shape + PLATE_ROOF_KILL +
   NORTH_S1 + RETURN_WALK + NW_KILL=1) completed the full mission (32562 ticks, stuck 0,
   falls 0). The committed defaults are now that shape via the `cd()` closure at the walk's
   head: every knob defaults to its proven value for the container roll (d3 real damage) only,
   the env override still wins, and the d1/d2/plain paths default 0 — byte-identical with all
   prior rounds. The committed no-env run reproduces the green roll exactly (32562 ticks).
   The measured-dead knobs stay 0 (RETURN_FAST, RETURN_EAST, NW_HEAD, NW_HEAD_EARLY,
   PHOTO_CROUCH).
7. The ignore attribute is REMOVED (the ledger text became the test's doc comments, with the
   green outcome appended). No `crates/*/src/` change, no `crates/nf-view/tests/common/`
   change, no guard code changed, no damage/health constants changed, no top-ups on the walk
   phases.

## How it was verified

- The runs (all [L], the direct test binary, --test-threads=1, each its own roll): 15-a
  t23386; 15-b t24481 (#6 +81); 15-c t23807 (the #4 plate act blocked, 260 ticks); 15-d
  t23831; 15-e t24481 (byte-identical to 15-b — the free act); 15-f t22911 (the aim-phase
  burst); 15-g both return legs 1200-tick stuck at (139.15, −311.65); 15-h the nav web stuck
  (crouched=true head_bump=true); 15-i the edge crossed +355, death t24050 on the walk_to
  second leg; 15-j death t24050 (the fast second leg, the same roll); **15-k GREEN — the
  full mission completed in 32562 ticks, stuck 0, falls 0** — and the committed no-env run
  GREEN identically (41.85 s).
- Whole binaries on the final tree (the committed defaults, no env): **walkreal_0700000d_live
  4 passed / 0 failed / 1 ignored (142.78 s)** — the 00agent walk runs unignored now (the
  hurt_census probe stays ignored) — and **interactables_0700000d_live 11 passed / 0 failed
  (29.72 s)**.
- Lesson-10 grep: the touched file is the 0d test file itself (`grep -l kill_guard_certain_at|
  kill_guard_index_certain|kill_guard_head_present|fast_leg crates/*/tests/*.rs` → only
  `walkreal_0700000d_live.rs`; no `crates/*/src/` and no `tests/common/` file changed), so no
  other binary is affected by construction; both 0d binaries were run whole anyway (above).
- The fast runner (`tools/test/suite.py`) was started once at setup (setsid, per the task) and
  this sandbox killed it between tool calls with `suite.out` empty — the rounds-1-14
  signature, reproduced again.
- **The suite ran per-binary in the foreground on the final tree (this sandbox held the
  build)**: the three non-view crates' whole suites — `cargo test --no-fail-fast -p
  nf-game -p nf-formats -p nf-audio -- --test-threads=1` — every binary green (nf-game 29,
  nf-formats 107+11+9+... , nf-audio's set — 151+ passes total) with ONLY the slim-ISO
  permitted failures: the three `disc_golden` binaries (crp_mesh_disc 2, disc_golden 3,
  identity_disc 1 — all instant 0.00 s misses, the missing cross-check dumps the slim ISO
  does not carry). The remaining nf-view test binaries' build then exhausted the rootfs
  (1.4 GB free, the rounds-13/14/15 ~5.5 GB four-crate wall) and was stopped — the whole 0d
  binaries above are the nf-view evidence.
- Live checks for the main agent: none required this round — the walk is green.

## What's left

- Nothing for this walk: the 00agent arrival-to-C5-door real-damage walk is green and
  committed, its ignore removed, the ledger preserved as the test's doc comments.
- The full four-crate suite (`tools/test/suite.py`) still needs a sandbox that keeps
  background processes alive between tool calls and a bigger rootfs (the rounds-13/14/15
  disk wall); the whole 0d binaries are the evidence this round.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures
for the session (no gauge, no billing panel, no API counters are exposed to the agent).
