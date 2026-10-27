# level-0a-band-settle-6-b — the bisect verdict: D3 (the Q2 re-run) moved 01's plate walk; the mis-encoding was the caller's flags-99 gate missing on the SECOND Dist2Tri call; the settle-5 radial tail landed (D2 held), the doorway slide reproduces the game's rate

## What was asked

Bisect settle-5's five radial-tail deltas against `walkreal_07000001_live`'s
`walk_from_the_spawn_to_the_exit_with_the_contact` (one env flag per delta), decide the 01-mover
from the decompile [M 0x109da0 / 0x10a53c-0x10a5e4], re-derive the 01 pins if the game's rule
really skips those faces (else fix the mis-encoded delta), pin the faces/ticks with the [L
2026-10-26] jamb push table, then un-ignore the three `walkreal_0700000a` tests with every
walkreal test green.

## The bisect (walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, one env flag each)

The five deltas re-landed behind `NF_CAPS_D1..D5` (all OFF = the settle-3 merged behaviour;
commit `level-0a-band-settle-6-b: the five settle-5 radial-tail deltas re-landed behind one env
flag each`), then:

| run | result |
|---|---|
| baseline (no flags) | **ok** (20.1 s) |
| D1 (radial-arm removal) | ok |
| D2 (first-query flags-99 skip) | ok |
| **D3 (the Q2 re-run)** | **FAILED** — `walkreal_07000001_live.rs:171`, the hall-guard AND (ch 0x99) never fired |
| D4 (the dot gate) | ok |
| D5 (\|v\|==0 → dir 0) | ok |

D3 alone broke 01 — the failure was the **hall-guard fight**: with D3 on, one of the named
guards (11) survived Bond's clearing (in the first repro the gun ran dry — clip 1 spare 0 at the
shot — because the walk's earlier fights consumed it differently), so the AND never fired.

## The decision [M — the decompile + mipsdis over work/elf/ACTION.ELF]

The second-query block is the game's [M Intersect_CylGeom 0x109da0, disasm 0x10a344-0x10a4fc]:
Dist2Tri #1's out-pointer is copy+0x50 (0x10a2fc `addiu t2,s1,0x50` before the 0x10a324 jal);
when its flags come back ∉ {0,99} the game projects copy+0x50 (= Q1) onto the spine A/B clamped
[0,1] (0x10a3b4/0x10a3c4) and RE-RUNS Dist2Tri from it (0x10a4fc, the out-write again) — the
tail's Q = Q2. The re-run itself IS the game's rule; settle-5's [L] (96(sp) riding the spine,
copy+0x60 constant) already refuted the accumulator-image reading. **The mis-encoding was the
caller's gate**: the game gates the push on `0.0 <= return && flags != 99` where BOTH values come
from the LAST Dist2Tri call [M 0x10a510 `c.olt` + 0x10a528 `beq v1,v0(99)` — the disasm loads the
flags word the call wrote; the decompile's `if ((0.0 <= fVar30) && (sStack_130 != 99))` tests the
second call's results too]. My first D3 encoding used the re-run's Q2 unconditionally: a re-run
whose projected query sits strictly behind the face plane returns flags 99 with the out-pointer
NEVER rewritten (it keeps Q1) and the whole face must stay inert — the port instead pushed
THROUGH such a face (v = query − Q pointing behind it), which shifted Bond's walk and the fight.

**Fix** (crates/nf-game/src/world.rs, the `on_edge` block): after the spine projection, the
re-run's input gets the same behind-plane test (`dot(p − t0, n) < 0` → the face is inert), then
the re-run. With the fix, **D3 alone runs 01 GREEN (12.9 s)**.

## What landed

The settle-5 radial tail is now UNCONDITIONAL in `capsule_tri_contact` (crates/nf-game/src/world.rs):

1. the settle-3 radial arm (ref = the HITTEST+0x50 accumulator) removed — dir/mag = (v̂, r − |v|)
   with v = query − Q [M 0x10a53c-0x10a61c]; the accumulator is only the add-back target
   (Collide_Intersect 0x1ebaa8:353-373);
2. the on-edge second query (project Q1 onto the spine clamped [0,1], re-run, Q = Q2
   [M 0x10a344-0x10a4fc]) **with the caller's gate on the second call** [M 0x10a510/0x10a528] —
   the bisect's fix;
3. the always-on dot gate [M 0x10a568];
4. |v| == 0 → the dir quad ≈ 0 [M 0x10a5bc `sq v0,0(sp)`].

**HELD BACK, env-gated (`NF_CAPS_D2`, default off): the first-query flags-99 skip** [M 0x109770 +
0x10a33c `beq v1,v0(99)` → 0x10a508]. It is [M]-faithful and green alone on 01, but its
combination with the landed tail WEDGES the 01 escort walk: after "hall guard 6: out of ammo",
every nav leg out of **(5.48, 6.67, 11.60)** gave up at the 3000-tick cap with
`segment_to_target=Some` on every target (the resupply pocket — a walk shape the old through-push
fiction used to carry Bond out of, and which the game's true skips remove). The next round
re-derives the 01 harness pins against it (the harness's resupply corridor logic is the brittle
part — a real player walks to the box). The combination `D2 + D3-without-my-gate` was settle-5's
exact landed code; with my gate the wedge is the remaining D2 blocker.

## The [L] jamb table and the 0a slide

The through-line route fix (settle-5's decode: the [L] glide is ONE walk — the old
(−8.45, 16.29) stop sat inside the jamb column's x-range) is landed in
`the_west_hall_glide_height_matches_the_game`. With the landed tail the glide **crosses the
doorway at the game's rate: 155 ticks, exiting (−9.27, 1.12, 16.60)** — the same 155-tick
crossing settle-5 measured, matching the [L 2026-10-26] fresh-boot glide. The jamb push table's
rotation (x up to 0.061, z up to 0.048, zero from f15) is what the crossing now rides; a
per-tick `HITTEST+0x10` assertion inside the test needs the port's per-tick DeltaP exposed — not
built this window (open item below).

## The trio (0 of 3 un-ignored — each note now names its one remaining mechanism)

- `the_west_hall_glide_height_matches_the_game` — route fixed, slide reproduced; the y pin still
  reads **1.1201 vs the settled 1.1285 [L]**: the hall glide's +0.0256 climb settle (the
  feet-band walked-dy growth, band-settle finding 1) is the separate mechanism, unported.
- `the_corner_crosses_like_the_game` — still the floating wedge (STUCK at (−17.87, 1.20, 4.87),
  grounded=false): the below-B edge-lift unhooking the band at its entry height (settle-2's
  mechanism), unchanged by the tail.
- `walk_office_floor_qworm_keyboard_hatch` — the ring rides the same two mechanisms.

## What I ran

- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1` — **578 passed / 0
  failed / 5 ignored** (the landed tree).
- `walkreal_07000001_live --test-threads=1` — **3 passed / 0 failed** (the landed tree; the
  bisect table above is from the flag builds).
- `walkreal_0700000a_live --test-threads=1` — **5 passed / 0 failed / 3 ignored** (the landed
  tree), and the trio `--ignored` re-measured (the two failures above).
- The full suite (tools/test/suite.py) on the final tree — see "Suite result".

## Live checks for the main agent

1. **The resupply pocket (the D2 blocker)**: 07000001, walk the meet-hall detour toward the
   named guards with Bond's position logged per tick around (5.48, 6.67, 11.60) — does the game
   walk him THROUGH that cell, and which faces touch the capsule there (HITDATA+0x40/0x0c per
   tick)? This decides whether the port's upstream walk diverges before the pocket (my
   suspicion) or the game's skips genuinely leave a pocket the harness must route around.
2. **The glide's y-settle**: the +0.0256 climb (1.1201 → 1.1285) while sliding the jamb — the
   feet-band walked-dy growth per tick (BL+0x760/0x770 y, the feet band dy) would settle whether
   the climb is the walked-dy growth of band-settle finding 1 or a per-tick push y component the
   port drops.

## Suite result

(to be filled before the push — see the final section)

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
