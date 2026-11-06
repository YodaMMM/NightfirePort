# level-14-float-narrow — 14 hover: a narrower fix

Branch `s/level-14-float-narrow`, from `origin/s/level-14-float-on-master` 80f472e88 (the hover
fix + `probe14_pocket_settle` on master d6245281a).

## Verdict

**Not done.** The spine fallback's small up-facing push is real work for the standing capsule
(lifts, stair flights); at the crouched pocket it parks Bond one ulp above the grounding band.
Three fix shapes were implemented and A/B-measured; every one that grounds the pocket moves
other green pins. The tip is reverted to **master's collision behaviour** with the leading
candidate (`segment_hit_scoped_feet`, the game's plane-form feet-ray t) **present but
inactive**, so the branch costs master nothing and the next message resumes from a measured
ledger. The only suite red vs master is `probe14_pocket_settle` itself — the hover this task
was to remove.

## The measurement (task item 1)

A/B: the hold skip gated behind `NF_FLAT_HOLD` in one build, then the six gate binaries on
both settings.

| gate binary | master (push on) | base hold skip | crouch-gated skip | feet-form band |
|---|---|---|---|---|
| probe14_pocket_settle | FAIL (the hover) | PASS | PASS | PASS |
| fix_autoaim_elevator_live | PASS | FAIL (boarding) | PASS | PASS |
| walkreal_07000011_route_live | PASS | FAIL (flight C) | PASS | PASS |
| mp_net_slots_post_move_live | PASS | PASS | PASS | PASS |
| mp_perf_16_live (pinned digest) | PASS | FAIL | FAIL | FAIL |
| fix_npc_doors_live | PASS | FAIL | PASS | FAIL |

**Where the small up-facing fallback push fires (measured, `NF_CAPS_LOG=1`):**
- **Lift boarding (0700004a, standing cc 1.0347, B = pos − 0.4847):** 43 HOLD-SKIP events during
  the 60-tick boarding, on the platform top (`n = (0,1,0)`, d 0.5500, mag ≈ 0.0000x) and its
  sloped skirt (`n = (0, 0.835, −0.550)`, d 0.5435). Without the push the radial arm's limit
  cycle dithers Bond at dy ≈ 1.145 — **outside the standing band edge cc + 0.1 = 1.1347** — and
  `grounded` flickers false; the boarding assert fails. With the push (master) the spine
  fallback parks him at the r-equilibrium dy ≈ 1.032 — **inside the band** — stably grounded.
  The push is the port's stand-in for the game's walked floor ride at the rest height
  (dy 1.0891 [L, band-settle finding 1] — the same zone).
- **11 stairwell flight C (standing):** 6964 HOLD-SKIP events on the descent; the descent rides
  the stair-edge fallback pushes ("the stair-edge rolls"). Skipping them stalls flight C at
  y 32.5025 > 32.5 (the descent leg's band check).
- **The 14 pocket (CROUCHED cc 0.55, B = pos):** the same push class fires mag 0.00272 every
  contact tick and parks dy at **0.6500001 = 0x3f266668** — the band `(dy·dy).sqrt() − 0.1 ≤ cc`
  needs dy − 0.1f32 to round to ≤ 0x3f0ccccd; 0x3f266668 − 0.1f32 rounds to 0x3f0cccce, **one
  ulp over cc**. The game's crouched rest edge is 0x3f266667 [L, the vent rest: "hit − 0.1f32
  rounds to exactly cc"], so the game's ride catches and the port's pushed ride misses by that
  one ulp. Measured dither (master behaviour, `NF_WALK_DBG=1`): grounded tick at dy 0.64727783
  (band TRUE) → the push lifts +0.00272 → dy 0.6500001 (band FALSE) → falls back — forever;
  `grounded_history` never reaches 0xf.

**Why the pocket differs from the lifts/stairs:** the standing capsule's push equilibrium
(dy 1.032) has ~0.1 of band slack (edge 1.1347); the crouched pocket's equilibrium (dy 0.6500001)
sits exactly AT the band edge (0.6500000134) and the port's contact arithmetic parks it one ulp
above, where the standing cases have slack to spare. The push is doing the same job everywhere —
only the crouched geometry leaves the equilibrium outside the band.

## What stops Bond grounding at the pocket (task item 2) — the ulp's source

The band arithmetic matches the game [M `Player_FeetOnPoint` 0x194188:106 `HITDATA+0xC − 0.1 ≤
cc`; the ray from obj+0x30 to HITTEST+0x30 − up·(cc+0.6) :89-100 — the port's ray shape is
identical]. The divergence is upstream: the game's `Intersect_RayGeom` 0x10a8c8 computes
**t = −planeDist(from)/dot(dir, n) — one division** [M :176-185] and HITDATA+0xC = |from − hit|
by sqrt-of-squares [M :312-325]; the port's cast is Möller–Trumbore (`tt = dot(e2, q)·inv`),
whose rounding differs by ulps. At the pocket the MT t reads dy 0.6500001 (0x3f266668, fails);
the plane form at the same contact reads dy ≈ 0.6499996 (passes with 2 ulps to spare).
`segment_hit_scoped_feet` (in the tip, inactive) implements the game's plane-form t for the
feet ray only; switching the two band call sites to it made the trio green **but moved the
mp_perf_16 pinned digest** — the feet ray feeds the MP sims, so a band-wide arithmetic change
cannot ship without re-pinning, which the task forbids.

## Fix variants tried (all committed in history)

1. `834ee5974` — master collision restored + feet ray plane-form t (feet band only): trio green,
   npc_doors FAIL, mp_perf_16 digest moved. [M-faithful, blocked by the digest pin]
2. `b2f78c6c8` — the base's flat-hold skip CROUCH-GATED at the two walk call sites: 5/6 gates
   green but the crouched regime is common — master A/B on walkreal_07000004_live: 10/0 green on
   master, 8/2 with the gate; same-class reds on 01 (2), 05 (1), 06 (2), 0c (1), 11 (3), 12 (1),
   13 (2), 15 (1), 16 (1). Not shippable.
3. The base's always-on skip (80f472e88): the task's own premise — elevator/11/npc_doors/MP
   digests red.
4. The tip `126d3fafa` — master behaviour, `segment_hit_scoped_feet` present but inactive.

## How this was verified

- The six gate binaries on every variant (the table above), `--test-threads=1`, NF_ISO set.
- The walkreal set partially, on the crouch-gate variant and on master: 24 binaries run
  (`/tmp/walkreal_results.txt` ledger in-session): 0b 7/0, 0a 13/0, 09 8/0, 02 9/0(?), 07 5/0(?),
  08 (timeout — pre-existing slowness), 1b 17/0, 41 15/0, plus the reds listed above. The full
  69-binary set did not fit the session; the tip equals master's behaviour so the master suite
  state applies to it by construction (the only code change is the inactive `segment_hit_scoped_feet`).
- `cargo test --offline --release -j2 -p nf-view --no-run` clean (0 errors) on every variant.

## What's left

1. Ground the pocket without touching the shared band arithmetic: the next candidate is the
   **contact-dist arithmetic** in `capsule_tri_contact` vs the game's `Dist2Tri` 0x109770 — the
   push equilibrium's 1-ulp height comes from `dist = |p − q|`, not from the band. If the port's
   dist matched the game's Dist2Tri shape, the equilibrium should land at/below 0x3f266667 and
   the band catches with the push intact — but the same digest caution applies: measure the MP
   digests first.
2. Or scope ANY fix to leave the MP paths bit-identical (the digest pins the current port
   behaviour; the task forbids re-pinning).
3. Then: the six gate binaries, the walkreal set whole (69 binaries), the full suite, the MP
   digests unmoved.

## Live checks for the main agent

1. The pocket rest height on PCSX2: poke Bond crouched at (33.97, 0.60, 30.95) in 07000014,
   let him settle, and record BLData root y and BLData+0xcc — expect the crouched rest to
   settle with `HITDATA+0xC` (the feet-ray hit distance) reading **0x3f266667** (the [L] vent
   rest edge), not 0x3f266668.
2. The lift boarding (0700004a): with the debugger on `Player_FeetOnPoint` 0x194188's band
   compare (:106), confirm the standing ride's HITDATA+0xC during the boarding ticks — expect
   ≤ cc + 0.1 (≈ 1.03-1.09), grounding via the band WITHOUT any capsule push (the game's tail
   is the radial arm alone [M 0x109da0:431-447]).
3. `HITTEST+0x50` (the DeltaP accumulator) during a stair descent on 11 — the port's radial arm
   uses `ref_v` = the lifetime accumulator [L boots (0,0,0), drift ≈15 m]; a live read of its
   walked value would settle whether the game's radial tail can ever fire on a walked stair line.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
this session.
