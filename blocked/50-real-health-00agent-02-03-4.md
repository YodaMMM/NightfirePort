# real-health-00agent-02-03-4 — the 02 and 03 00 Agent walks (low priority)

Continue from master and read docs/reports/real-health-00agent-02-03-3.md on origin/s/real-health-00agent-02-03-3: the 03 death is diagnosed and the pre-clear kills the decisive cluster; one approach problem and one stall remain. The 02 sister was not started. Real input only; no top-ups; helpers opt-in. Remove each ignore when green; every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox3 at 2026-11-06T05:09Z

---
## Report (blocked, 2026-11-06T05:36Z)


---
## Report from the worker (blocked, 2026-11-06T05:36Z)

# real-health-00agent-02-03-4 — report

Branch `s/real-health-00agent-02-03-4` (cut from `origin/master` c2c4a6efd). The tree touches
ONLY `crates/nf-view/tests/walkreal_07000003_live.rs` (03-side, all `difficulty >= 3`-gated) —
**no `crates/*/src/` change, no shared-helper change** (`walker.rs` untouched), so every other
test binary is byte-identical to master by construction.

## Status: the 03 walk is RED with the death moved and the mechanism mapped; the 02 walk is
## RED (reproduced, the mechanism named); both keeps keep their `#[ignore]`

## What was done (all in the 03 file, difficulty-3-gated)

1. **Repro on master** (`--ignored --nocapture`): the 03 00 Agent walk dies at **t222034 at
   (21.94,6.51,7.73)** — 28 hits for 100: the mid-courtyard #9 fight 32.4 (9 hits at d 29.8 —
   at d1 the same fight costs ZERO, at d2 2.1: the DroneFiring_Accuracy 0.5→1.0 scaling lands
   everything at 00 Agent), the west lawn 31.2 (#17/#12 single 12.0s, #8 7.2 killed-false),
   and the death burst 36.4 — **the point-blank shotgun post #5 (22.8,6.3,9.8) d 2.3: 31.6 in
   8 hits** (3×7.2 pellets a volley) plus #4/#3 — Bond died in ~90 ticks, too fast for any
   fight to answer (9 fights total, none engaged #5). The run also carries ~140k ticks of
   east-lane/door stalls (t16330 → t156454 wedged).
2. **The d1/d3 nav diff (the key finding)**: from the SAME 0x50-plate stand (24.5,5.47,−13.66)
   the d1 nav takes the EAST line (first node (34.95,−4.07), door 8) while the d3 nav takes the
   north wing and walks Bond into the post. The web's cross-path hops run `move_test` against
   the world **doors included** [M walker.rs `nav_route_global`; door.rs `CLOSED = 2`] — and the
   run-4 door dump [L] shows **doors 4/5/6/9/10 ALL read CLOSED at the d3 leg start** (door 6
   (26.6,−14.6) sits on the direct line). A closed leaf flips the route.
3. **The corridor pre-clear + door-opening attempts (runs 5-7)**: opening the east-line doors
   (walk-to-centre, the walk's own E arm) exposed the NEW cluster position — under the merged
   master the door-8 cluster HOLDS THE EAST CORRIDOR: **#2 (the captor, 0x5000038 shotgun) at
   (29.1,6.2,−13.1) d 3.4, #19 (0x500003a) at (27.2,−13.5) d 1.8**. The fights answered and
   **killed #2** but the exchange cost 34.0 (36.4 → 2.4) and #19's finishing blow landed at
   t217995. A pre-clear from the plate stand finds nobody (the guards close in AFTER the door
   noise — the box was empty at every pre-walk moment I tried); after the door-6 walk the door
   never opened (the walk arrives within r 1.3 before the E arm fires), so the pre-clear behind
   it still finds nobody.
4. The sidestep ladder and the east anchor were added around the servant-0 walk — they never
   fire because the death happens INSIDE the first `walk_nav_to` (its internal nav-2
   re-derivation walks north before the call returns).

## The 02 walk (reproduced, not fixed)

`walkreal_07000002_live walk_the_castle_all_objectives_real_health_00agent`: dies at **t6833
(−31.35,6.46,8.40)** — 66 hits for 100, of which **#7 (0x500003a, weapon 0x1c shotgun) at d 3.5
landed 88.8 in 32 hits** — the geisha-1 shotgun captor, fought with a DRY gun ("[02] geisha 1
stand-off 0 shotgun: 0x1c not carried"; the cuffs 0x47 at clip 0 spare 0). The scoop economy
(the walk's own 00 Agent tactic: take the dropped shotgun, one 8-pellet tap kills a 12-hp
captor) found no dropped 0x1c at the moment it needed one. The next session's lever: reach the
geisha-1 room with a carried shotgun (an earlier captor's drop scooped deliberately), or re-order
the stand-off so the drop lands before the duel.

## What's left (the next session's levers, in order)

1. **The 03 corridor cluster**: open door 9 FIRST (from cover — the door-9 walk at (32.2,−13.3)
   is what triggers the engagement), or pre-clear the corridor with a WIDER box run right after
   the door-9 walk's noise pulls the cluster in; the fights kill #2 at zero further cost once
   the exchange starts — the walk needs to START it from full-ish health (the mid-courtyard #9
   fight's 32.4 is the pool hole: a pre-emptive kill of #9 from the courtyard-east stand before
   his first volley is the cheap cut).
2. **The 02 geisha-1 shotgun economy** (above).
3. **The 03 exit lawn** (the -3's levers 2-3: the (35.6,8.7) door stall, the #16 post-kill
   volley) — still unexercised at d3; the d1 run passes them with 12.8 left.

## How it was verified

- 7 runs of the 03 00agent test this session (runs 1-7, each ~95-100 s, deterministic per tree):
  master repro t222034 → the ladder/anchor runs (identical, the death inside the first nav) →
  the door-dump run (same death + the door table) → runs 5-7 the corridor shape t217995.
- The d1+d2 runs of the same binary: **2 passed** (`..._real_damage` and `..._real_damage_agent`
  both green on master and on this tree — the difficulty-3 gating keeps them byte-identical).
- The 02 00agent repro: 1 run (t6833, quoted above).
- The full-suite total line and the every-`walkreal_*_live` sweep did not fit the window
  (~90 min went to the seven 03 iterations + the 02 repro); no `crates/*/src/` or shared-helper
  change exists, so the other binaries are unaffected by construction.

## Live checks for the main agent

1. The 03 east-corridor doors (6 (26.6,6.6,−14.6), 9 (32.2,6.5,−13.3)) on a real PCSX2 00 Agent
   run from the servants' ground: does the game's own Bond open them, and do the door-route
   guards (#2/#19/#18) hold the corridor when he does? A fieldprobe E at door 9 from (26,−13)
   would settle whether the corridor is the game's own path to servant 0.
2. The 02 geisha-1 drop: does captor #7 drop a 0x1c at death (the walk's scoop reads no drop
   while he lives)? A gdbwatch on the drop record when #7's health hits 0 would confirm the
   scoop's timing assumption.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
