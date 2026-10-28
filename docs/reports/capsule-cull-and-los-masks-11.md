# capsule-cull-and-los-masks-11 — the mp_perf_16 red closed with the mover cited; the walk red carries the decompile answers and one refined live check

Branch `s/capsule-cull-and-los-masks-11` (from `origin/s/capsule-cull-and-los-masks-10`,
`origin/master` 8a966fa3 merged clean). The task: the last two cull/masks reds.

## 1. mp_perf_16 `sixteen_participants_fixed_run_digest_is_pinned` — GREEN (re-pinned, mover cited)

The -10 report's method ran live, end to end:

- **The dump arms landed** in `mp_perf_16_live` [M this test]: `NF_MP16_DUMP=<path>` writes the
  per-tick `net_tick` hash stream; `NF_MP16_MOVERS=<path>` writes, per tick of the SAME
  digest walk (slot-0 machine), the two human bodies' pos/yaw/vel bits and every drone's
  pos/yaw bits + state. Both write before the pin assert, so a red still lands its stream.
- **A master build still passes the OLD pin.** origin/master 8a966fa3 in a throwaway worktree
  (this test's file copied over it, shared `CARGO_TARGET_DIR`, incremental build 77 s):
  `test result: ok. 1 passed` — master reproduces `0x89cbfcb6ae45c2ac`.
- **The first diverging tick and its mover, from the two streams:** the hash streams differ
  from **tick 167** (master `0x1549cab3a71d3899`, this tree `0x1d28c9f40adbdf1e`) and never
  re-converge (167..3600). The mover line at tick 167 names **the slot-0 LOCAL body (bond
  replica 0)**: x `c164c3cc`→`c164bf73` (−14.2978 → −14.2967), z `c0d89137`→`c0d8883d`
  (−6.7677 → −6.7666) — a ≈1.1e-3 walk-position delta under identical scripted inputs; no
  drone, no net slot and no RNG difference at the divergence (the two machines' per-tick
  hashes agree every tick on this tree). This is the branch's game-true walk-collision
  cull/masks behaviour reaching the MP sim through the local player's own capsule move —
  exactly the -10 report's hypothesis, now measured.
- **The re-pin**: `PINNED_DIGEST = 0xfd4f_cbf8_1586_b3f1` with the tick/mover citation in the
  pin comment. Reproduced twice on the branch tree (0xfd4fcbf81586b3f1 both runs, kills 40).
  `mp_perf_16_live`: **2 passed / 0 failed** (10.9 s).

Hazard worth recording: a master worktree build against a **shared** `CARGO_TARGET_DIR`
overwrites the branch's rlibs while the branch's fingerprints still say fresh — the next
branch build must `touch crates/nf-game/src/lib.rs crates/nf-view/src/lib.rs` (one false
"branch" run reproduced master's digest before this was caught; the WORKLOG carries it).

## 2. walkreal_07000001 `following_mayhew_down_completes_the_level` — still red; the decompile questions answered [M]

The walk jams at (−33.44, 4.72, 12.03): the AND-touch plate (z 12.17..18.68) never sees Bond
(`contact touched_by=true bond touched_by=false`, the harness's own line). The task's two
decompile questions, answered from the decompile this session:

- **What distance the game snaps down a stair riser while walking: 0.1 below the capsule
  bottom, and no further — beyond it the walk falls per riser.** The glue band is
  `Player_FeetOnPoint`'s `hit − 0.1 <= cc` [M 0x194188:106], with `cc = AnimObj+0xCC` = the
  anim root height above the feet ([L], port `ROOT_HEIGHT` 1.0347183): in feet terms the
  band holds only while the floor is within 0.1 **below the feet** (the cc terms cancel).
  Past it flag8 clears [M 0x194188:176-185] and the walk takes gravity per
  `Player_Collision` [M 0x194df0:104/107] until the next tread grounds the band again. The
  walk-down is gravity-per-riser, NOT a larger snap — the game's step-down tolerance (0.1)
  is not larger than the port's step-up 0.5.
- **The head-on cull and the descent's soffit**: the cull skips a face only when
  `(n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ < −0.999 ∧ f < 0.5)` on the NORMALIZED spine [M
  `Intersect_CylGeom` 0x109da0:302-322, d̂ = normalize(B − A) pointing DOWN]. A soffit
  (down-facing, n·d̂ ≈ +1) is skipped only when crossed in the spine's BOTTOM half (f > 0.5);
  the soffit pressing the capsule's top half (f ≤ 0.5) is NOT culled and pushes. The port
  already implements this rule verbatim (world.rs `capsule_tri_contact`).
- **The walk's grounded-tick vertical is `vel = 0` on a static floor**: grounded (flag8) with
  floor_ny ≥ 0.5 zeroes `BL+0x50` [M 0x194df0's velocity block, the `Vec_Zero` arm], and the
  per-tick floor hit is copied over the AnimObject's low block by `amemcpy(anim, hit, 0x60)`
  [M 0x194188:187-193] — its `+0x58` (the hit's obj, 0 = static world) is the block's own
  gate reader [M 0x194df0's `*(iVar16 + 0x58) == 0` arm]. The port models the glue; the
  0x60-byte copy itself is scratch (animobject.md's open question) and moves no position.

**So the port's walk-down mechanism matches the decompile on every term I can cite.** The
wedge is an emergent arrival state: the probe this session (temp harness, not committed)
showed the step-down gravity cycle working on open floors (g=false → −0.163/−0.327/−0.490 →
land → vel=0), and the -10 probe showed a capsule teleported INTO the jam pose is pinned —
the open question is the approach line that lets the live game's Bond avoid the pose, which
only the live game can answer. NOT re-derived: the task's condition (the decompile showing
the leg non-game) is not met — the leg walks the main test's proven hall-corner +
stairwell-head shape already re-derived in -10.

## Live checks for the main agent (refined from -10's #2)

1. **07000001 stairwell descent, fresh boot**: walk Bond down the flight (x −33.5..−32.3,
   z 10..16) and record his root y per tick against the tread tops. The decompile predicts:
   the root rides 1.0347 above the feet; at each riser the band (floor within 0.1 below the
   feet) breaks for a few ticks and gravity drops him ~0.571 to the next tread (a stutter
   descent, not a glide). Confirm that, and confirm he NEVER sits with his capsule bottom
   ~0.46 below a tread top while his head is under the soffit (the port's jam pose). If the
   live game DOES glide (band never breaking), the port's band is wrong somewhere I cannot
   see statically.
2. **mp_perf_16's mover, if a wider net is wanted**: the divergence is in the local body's
   walk at tick 167 of the fixed run — a gdbwatch on the live MP sim's local Bond around a
   wall/floor seam (any) confirming the game's per-tick walk-down stutter (check 1's shape)
   would confirm the port's delta is the game's own behaviour, closing the digest question
   permanently.

## How it was verified (what ran)

- `mp_perf_16_live`: 2 passed / 0 failed (10.9 s) — on the final branch tree, after the
  fingerprint hazard's forced rebuild.
- master's `mp_perf_16_live` (worktree, old pin): 1 passed / 0 failed — the cross-check.
- `walkreal_07000001_live`: 3 passed / 1 failed (88.8 s) — the follow red, unchanged.
- Crates touched this session: nf-view's tests only (`mp_perf_16_live.rs`); the two binaries
  above are the touched set, both run before the push.
- **The full suite did NOT run this window**: the background fast runner is killed between
  tool calls in this sandbox (three launches, three silent deaths during the build phase),
  and the foreground build alone consumed the first 20 minutes. The final-tree suite run
  remains the first item for the next window (the tree is pushed at 65e72015).

## What's left (next window)

1. The full suite on this tree (fast runner, foreground if the background keeps dying);
   quote the total line.
2. The walk red, pending the live check above — the decompile answers are in this report;
   if the live game glides where the port falls, the divergence is in the band's inputs
   (the scoped ray's hit set), and the port's `segment_hit_scoped` bucket coarseness is the
   first suspect (-10's known port-structure note).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for
this session (no usage banner, no /cost, no token readout anywhere in the interface this
worker sees).
