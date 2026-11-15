# level-0d-operative-walk-10 — the ground-answer lever, measured dead; the ledger + the flight-trace next lever

Branch `s/level-0d-operative-walk-10`, base origin/master 820819cb3 (36db1e785, which carries
rounds 7-9, is an ancestor). The driver's lever: answer #29/#27 before the climb, from a line
the scans already found. The walk
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage` stays **ignored/red**; the
round-10 ledger and the next lever are in the ignore message.

## What's done

1. **Fresh recon on this tree** (two probe runs, assertion-free, `--ignored --nocapture`):
   - `probe_0d_ray_stands` (the existing matrix probe): every ground stand's eye→chest ray to
     the trio's LOAD placements (145.6-145.8, 22.70) dies 0.83-0.90 — the slab edge seen from
     below. The ground CLEAR lines the round-9 ledger names exist only to the SETTLED spots.
   - `probe_0d_ground_wake` (NEW, committed): wake at the drop point + a 700-tick settle, then
     (a) the 14 route-stand matrix, (b) `scan27`, the full 186..216 × −240..−222 grid at 2 u.
     Findings: the settled roof snipers stand at **HEALTH 10 each** (one aimed hit); the grid
     holds **56 CLEAR spots, ALL to g29** (x 204-216, z −234..−240: trench a (210,−240) d 59
     and trench f (214,−245) d 63, stable across two rounds 400 ticks apart); **g27 has ZERO
     ground spots** (her south spawn's clear-ray law is the ladder-top wake dump only); g28 is
     absent from the parked timeline (the run's strip-mouth #10 kill shot is what wakes her —
     she was present at the trench moment in both real runs).
2. **The lever wired and judged on its own real runs** (the legs were `real_damage && difficulty == 1`
   gated; the plain replay and the d2 twin untouched):
   - Run 1 (the trench-a act, cap 450): the leg walked +79 ticks, the census caught the whole
     trio (g27 (151.35,−255.80) 0x97, g28 (151.43,−241.25) 0x94, g29 (151.39,−245.26) 0x93,
     health 10 each), the g29 ray full CLEAR / static clear / bullet_scoped CLEAR at d 59.6,
     **two aimed shots fired — ALL MISSED** ("still alive after 450 ticks (health 10, dist
     60.0)"). #29 died anyway to the ladder-top FIRST pass's crossing catch (+222, the walk-9
     law; her hp 0 at frame 3854). Death **t5132** (the crouched baseline t5532 on the
     re-rolled dice; #27 3 × 16.0 from her south spawn the top killer).
   - Run 2 (+ the trench-f #28 attempt, cap 300, the same measured-blind stand): the trench-a
     act missed again (identical deterministic shots), the FIRST pass caught its guard at
     +222, #16's deck fire landed 32.0, the phase's later acts never ran. Death **t4336,
     WORSE** — the walk-6 act-fire dice law held: every added pre-climb act re-rolls the
     stream lethal-earlier (5532 → 5132 → 4336). 10 hits for 100.0 both runs, 0 return-fire
     fights.
3. **The legs are REMOVED** (the #18-leg precedent: the net is NEGATIVE) with the measured-dead
   law in the route comment; the route is the walk-9 shape again.
4. **The audit run** (the reverted baseline + `NF_0D_AUDIT=1`, read-only): death **t5532
   EXACTLY** — the walk-9 baseline restored deterministically (the revert verified). The audit
   confirmed the ladder-top torso bullets LAND on the trio (the g29 crossing kill: SPAWN part
   Some(4,0.836) world CLEAR, DEAD trv 52.2, the world stop BEYOND her). So the far-target
   miss is **specific to the GROUND up-hill lines** (eye ~1.8 → y 23.0 at d 59-63, a ~21° ray).

## How it was verified

- `walkreal_0700000d_live` whole (final tree): **2 passed** (`walk_arrival_to_the_c5_door`
  — the plain replay — and `walk_arrival_to_the_c5_door_real_damage_agent` — the d2 real-damage
  twin, both byte-identical), **1 ignored** (the real-damage walk with the refreshed ledger).
- `interactables_0700000d_live` whole: **11 passed, 0 failed**.
- The probe binaries: `probe_0d_ray_stands`, `probe_0d_ground_wake` (14-stand matrix + scan27,
  two rounds each), plus the NF_0D_AUDIT walk run.
- **The full four-crate suite did not fit the window.** The sandbox was reset before this
  session (cargo, 7z and the ISO were gone), so the setup + the release build consumed most of
  it, and background jobs die between tool calls here (the suite runner cannot survive in the
  background). This branch touches ONLY two nf-view test files
  (`crates/nf-view/tests/walkreal_0700000d_live.rs` + the new
  `crates/nf-view/tests/probe_0d_ground_wake.rs`) — no `crates/*/src/` changes, no
  `crates/nf-view/tests/common/` (shared-helper) changes — so the other levels' walks and the
  engine crates are untouched by construction. The two binaries the task names were run whole.

## What's left (the ignore message's next levers, in order)

1. **The trench-a bullet audit at d 60** (`NF_0D_AUDIT=1`, read-only, no draws — the walk-8
   tool): re-add the trench act under the audit env in a PROBE-STYLE run (never a shipped leg)
   and trace the two shots' flight. This session's audit already proved the ladder-top torso
   bullets land, so the suspect list is narrow: the flight's per-tick START-CEL re-feed
   [M Bullet_update 0x12f0cc-0x12f0d8] crossing multiple cels on the steep up-hill ray, vs the
   posed boxes the aim gate sees past. If the stop is the port's boxes-only refine missing the
   origin-centred two-sphere capsule [M NDrone2_Collision 0x140e90:91-121], the fix
   un-act-proofs the WHOLE far-target family (the trio from the top, #10 in the bay, #17's
   band).
2. The port-side capsule fix itself [M NDrone2_Collision 0x140e90:91-121] (its own task).
3. The yard levers (the fast first tap, the girder sweep, walk-5's wired set).

## Live checks for the main agent

- Watch one trench-a shot's flight in PCSX2 (fresh boot, the same stand (210.6, 1.1, −238.7),
  aim at g29 (151.4, 23.0, −245.6)): does the real game's bullet reach the box? If the real
  game lands the d-60 up-hill shot where the port's dies, the port's flight path is the
  deviation and the trace will show the cel where it stops.

## Usage

Token, context-window and cost figures for this session: **not visible to me** — this
environment exposes no token counter, context gauge or cost readout. Nothing estimated.
