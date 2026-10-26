# level-0a-band-settle-4 — Intersect_CylGeom's per-face DeltaP decoded at the metal (the LQC2/SQC2 flip, the Dist2Tri out-write into copy+0x50, the on-edge second query); the port experiment measured-and-reverted; no code landed (the tree = settle-3 byte-identical)

## What was asked

Port `Intersect_CylGeom` 0x109da0's per-face DeltaP exactly (decompile lines 300-460, the
edge/vertex regions included) so the port's pushes reproduce the driver's [L] 2026-10-26 jamb
frame table (0700000a, god mode, yaw −1.5708, stick forward from (−8.0, 1.12, 16.29); the
write-watch found only `Collide_Intersect` 0x1ebaa8+0x644 (PC 0x1ec0ec) writing HITTEST+0x10 —
the slide is the per-face DeltaP add-back), then un-ignore the three `walkreal_0700000a` tests.

## The decode [M — the disassembly, tools/crosscheck/mipsdis.py over work/elf/ACTION.ELF]

1. **mipsdis's VU0 store/load opcodes are SWAPPED.** Opcode 0x36 (printed `sqc2`) is MIPS
   **LDC2 = LQC2 — a LOAD**; opcode 0x3E (printed `?? op=62`) is **SDC2 = SQC2 — a STORE**.
   Every `sqc2 vfN, off(reg)` line in my cylgeom.asm dump is a LOAD; every `?? op=62` line is
   the real store. With the flip the radial block's memory traffic matches the decompile's
   `_lqc2`/`_sqc2` pairings exactly.
2. **Dist2Tri's out-pointer is copy+0x50 in BOTH calls** [M disasm 0x10a2fc and 0x10a4f0:
   `addiu t2,s1,0x50`; the call sites 0x10a324/0x10a4fc jal 0x109770]. The closest triangle
   point Q is written INTO the HITTEST stack copy's +0x50 — **clobbering the accumulator
   image** that `RotPreTransVec` stored there [M Collide_Intersect 0x1ebcc4].
3. **The radial block's true operands** [M 0x10a53c-0x10a554]: `lqc2 vf3,96(sp)` = the QUERY
   point (the plane crossing, or the centroid projection on the parallel branch — parked in
   96(sp) by the `?? op=62` delay-slot stores at 0x10a3bc/0x10a3d4/0x10a3e8/0x10a4b0/0x10a4c4/
   0x10a4d8); `lqc2 vf1,80(s1)` = copy+0x50 = **the closest point Q (the out-write!)**;
   `lqc2 vf2,0(s1)` = the face plane equation. So the decompile's `v = hitpoint − HITTEST+0x50`
   [:424] is really **v = Q − query** — the separation from the spine query point to the tri
   point, i.e. **the port's spine-rule separation vector, direction included**.
4. **The gates and the push** [M decompile :431/:447, cond codes calibrated: 4=olt, 6=ole,
   2=eq]: skip when `dot(v, n) <= 0` (0x10a568 c.ole f0,0.0) or `|v| >= radius` (0x10a5c8
   c.ole radius,|v|); push = v̂·(radius−|v|); `DeltaP` stored at 0x10a5e8 (`?? op=62` →
   0(v1), v1 = 0x2d9bd0 = DeltaP); the spine start/end += push (0x10a614/0x10a61c stores to
   copy+0x20/+0x30). `DeltaP`'s init = the prologue `sq v1,0(&DeltaP)` (0x109e1c) with
   v1 = \*(HITTEST+0x88) = the geom pointer — as float bits a denormal ≈ 0: the game's "clear".
5. **The on-edge second query** [M :333-364 + 0x10a344-0x10a4fc]: when the first Dist2Tri's
   flags ≠ 0/99, the game projects copy+0x50 — at that moment the RESTORED accumulator image
   (reloaded at 0x10a350) — onto the spine, clamped [0,1] (0x10a3b4 c.olt f20=1.0; 0x10a3c4
   c.olt f0,f23=0.0), and RE-RUNS Dist2Tri from it (0x10a4fc, query in 96(sp)). For the walk's
   vertical spine the accumulator's y 0.045 [L frame table] clamps the projection to the spine
   BOTTOM = the feet. The floor's no-push falls out: the crossing inside the tri (flags == 0)
   takes no second query, Q = the query itself, v = 0, the dot gate `0 < 0` fails — the floor
   never pushes [M], riding the feet band as settle-2/3 established.
6. **The [L] magnitudes check out**: the wall-press equilibrium push = the walk step
   (0.1/tick [L], acc 0.277→3.277) with the query pressed r−0.1 = 0.45 inside; the first touch
   r−0.539 = 0.01056 [L]; the jamb pushes 0.02-0.07 with the rotating x/z split = the
   separation rotating around the jamb edge as the feet pass.

## What landed (and was reverted)

One experiment: the port's on-edge re-projection (world.rs `capsule_tri_contact`, the line
`if on_edge { p = closest_on_segment(*a,*b,q); }`) replaced by the game's structure — project
`ref_v` (the accumulator image the walk already plumbs) onto the spine clamped [0,1] and
re-run the closest-point query from it. Measured:

- unguarded: `walkreal_0700000a_live` regressed — `west_doorway_repro` failed (the mission
  died at "ring: east of the west door" — the floor-edge contacts changed);
- guarded to non-horizontal faces (|ny| ≤ 0.7): the 0a battery green again (5 passed / 0
  failed / 3 ignored) but the three target tests (run with `--ignored`) still failed —
  the guarded fix alone does not reproduce the slide.

**Reverted**: `crates/nf-game/src/world.rs` is byte-identical to settle-3's merged state
(the previously verified green tree). The remaining divergence is elsewhere — the prime
suspects for the next round: (a) the parallel-branch query target (the centroid projection
vs the game's exact chain), (b) the +0x60 vector's role (the add-back shifts it too [M
0x1ebaa8:361-363]; the [L] center = pos+Σpushes model says the radial center rides it), and
(c) the 0.0002/parallel branch split for the walk's vertical spine — with the LQC2/SQC2 flip
the whole tail should now be re-decoded op-by-op and re-pitted against the [L] frame table
via `caps_log()`.

## What I ran

- `cargo test --offline --release -j2 -p nf-view --test walkreal_0700000a_live --
  --test-threads=1`: 5 passed / 0 failed / 3 ignored (the experiment, guarded variant), then
  the same trio with `--ignored`: 0/3 (still failing) — after which the code was reverted.
- The reverted tree was NOT re-run: it is the exact settle-3 merged code whose battery record
  is in docs/reports/level-0a-band-settle-3.md (walkreal 0a 5/0+3i, 01 3/0, 05 8/0, nf-game
  925/0). No source file differs from that state this round (git status clean apart from
  scratch).
- The background `tools/test/suite.py` start died in this sandbox between tool calls (it
  could not find `cargo` after the kill — the same no-suite-total situation settle-3
  reported); no suite total line this round.

## Live checks for the main agent

1. **The radial block's live query point**: at the jamb pass, dump the stack copy's +0x50
   (the value `lqc2 vf1, 80(s1)` loads at 0x10a540) for 3 consecutive contact ticks — the
   decode says it holds Dist2Tri's closest point Q (the out-write), NOT the accumulator
   image; confirm it reads near Bond's spine (within ~0.6 m), which pins finding 2.
2. **copy+0x60 across the same pass**: the add-back shifts it [M 0x1ebaa8:361-363]; dump it
   next to +0x50 — if +0x60 tracks pos+Σpushes (the frame table's implied center), the
   center-vs-accumulator question closes.
3. **The on-edge projection target**: the decode says the second query projects the
   accumulator image clamped to the spine (→ the feet for the walk). A breakpoint dump of
   96(sp) just before the 0x10a4fc call on a jamb contact tick confirms or refutes.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
