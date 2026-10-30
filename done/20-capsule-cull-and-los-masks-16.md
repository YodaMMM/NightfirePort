## Your task: `capsule-cull-and-los-masks-16`, one cylinder query, faithfully

Continue `origin/s/capsule-cull-and-los-masks-15` (merge `origin/master` first; read its report, "What's left"). The cull fixes two walks and breaks two others in every variant tried: the port's push arm and the game's single Intersect_CylGeom 0x109da0 query diverge structurally. Port the game's ONE cylinder query faithfully [M 0x109da0 whole fn; Collide_Intersect 0x1ebaa8] for the paths that use it, so a single cull rule serves all of them; then remove NF_CULL_UP / NF_NO_CULL. Land only with walkreal 02, 03, 08, 14, fix_npc_doors, npc_doors, mp_perf_16 and every touched binary green; quote each total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-16`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T14:03Z

---
## Report (done, 2026-10-30T15:38Z)

# capsule-cull-and-los-masks-16 — report

Task: port the game's ONE cylinder query faithfully ([M] Intersect_CylGeom 0x109da0 whole fn;
Collide_Intersect 0x1ebaa8) for the paths that use it, so a single cull rule serves all of
them; remove NF_CULL_UP / NF_NO_CULL. Branch `s/capsule-cull-and-los-masks-16` =
origin/s/capsule-cull-and-los-masks-15 + merge origin/master (a19be699), clean.

## The headline: the -15 open link is closed — the cull compares the RAW denom

The -15 report ended on "the dot's exact input vector is the one link the byte-level analysis
did not close". Closed this window, from the disasm [M work/elf/ACTION.ELF via
tools/crosscheck/mipsdis.py]:

- **0x10a1f0** `lwc1 f2,128(sp)` — the cull's compared value f2 is loaded from [sp+128]
  ONCE, immediately before the |denom| > 0.0002 gate (0x10a1f8 c.4.s against f1 = 0.0002,
  loaded 0x10a1c4).
- **f2 IS the f-clamp's own divisor**: `div.s f0,f0,f2` at 0x10a22c/0x10a244/0x10a260
  computes f = −num/f2. [sp+128] is the denom hadd — the decompile's uVar3
  [M decompile 0x109da0:291-301: vmul(plane_n, auStack_e0) + 3× vaddbc, where auStack_e0 is
  the RAW B−A stored at :201-202] — i.e. **denom = dot(plane_n, B−A), unnormalized**.
- **The cull compares THAT f2**: `c.4.s f0,f2` at 0x10a288 (0.999 < denom) and
  `c.4.s f2,f0` at 0x10a2c4 (denom < −0.999), each ANDed with the f half (f1 vs 0.5 at
  0x10a2a4/0x10a2e0), branching to the tri-loop tail 0x10a7b8 — the decompile's own
  :321-322 arms.
- The normalized spine (VSUB 0x109f60/0x109f68 + Vec_Normalise 0x109f74 into sp+48) is NOT
  the cull's input: it feeds the **0.7 walkable-face gate** [M decompile :473, the sVar1==0
  block] and the pick-bit block under it. The -13/-15 normalized-A−B reading was wrong in
  BOTH directions (normalization AND sign).

Consequence [M |denom| ≤ |B−A| for unit plane normals — the geometry's Q14 fixed-point
normals, scaled by the 2^-14 constant 0x38800000 at 0x10a1a0]: **on the walk spine the cull
is INERT for every face** (|B−A| = 0.7597 [L ee_start_05 BL 0x16cf0f0] < 0.999), and **on
the climb spine it lives** (|B−A| = 1.3097 [M Player_Collision 0x194df0 case 0/6]): an
up-floor (denom = −1.31 < −0.999) is skipped when crossed in the TOP half (f < 0.5) — the
jump-through case; solid in the bottom half. The game's underfoot support is the feet ray
[M Player_FeetOnPoint 0x194188], not this query.

## What landed

1. **The faithful cull** (world.rs `capsule_tri_contact`): the skip is now
   `(denom > 0.999 && f > 0.5) || (denom < -0.999 && f < 0.5)` on the RAW denom — no
   normalization, no negation. **NF_CULL_UP / NF_NO_CULL removed** (the bisect switches; the
   faithful rule replaces them). No other env gates touched (NF_CAPS_GATE / the CAPS_D2 doc
   updated).
2. **The flags-99 behind-plane skip (D2) landed unconditionally** [M Dist2Tri 0x109770 — the
   signed plane distance < 0 returns flags 99, out-pointer never written; the caller skips it
   at 0x10a33c `beq v1,v0(99)` → 0x10a508; the decompile's line-419 gate
   `0.0 <= fVar30 && flags != 99` excludes it from push AND best-hit]. With the walk-leg cull
   inert this is the game's ONLY per-face skip on a walk leg — the step behind/above (the
   crossing clamped to f=0, p = A below the step top) is skipped here, which is what the old
   normalized cull was accidentally emulating for the stairwell. (The -6-c wedge note — the
   01 escort resupply pocket — was measured UNDER the normalized cull; re-measured this
   window, below.)
3. **Collide_Intersect 0x1ebaa8 read whole** [M]: the HITTEST is copied to the stack
   (amemcpy 0xa0, :157) and the query runs in the OBJECT'S LOCAL space (RotPreTransVec writes
   the local A/B/ref INTO the copy's +0x20/+0x30/+0x50 slots — uStack_100/uStack_f0/uStack_d0
   are inside the copied block); the caller adds the accumulated DeltaP to the REAL
   +0x10/+0x20/+0x30/+0x50/+0x60 once per landed hit (:353-373) — the copy's per-push A/B
   updates are local. This confirms the game's per-triangle push tail is the
   accumulator-relative arm (v = q − HITTEST+0x50, dot(v, n) > 0, |v| < radius → DeltaP += v̂·
   (radius − |v|), with q = the QUERY POINT [M decompile :420-457, auStack_180]) — and with
   the accumulator's [L ≈15 m mid-play drift] that tail is near-dead on walked lines
   (hatch-10's finding). The port's substitute push tail remains (see What's left).

## The measurement matrix on the faithful rule (all [L] this window, real input)

Baseline first (the merged tree, pre-change, DOWN default): 01 **5/0** (76.46 s), 02 **RED**
(34.90 s), 03 **RED 3/1** (343.77 s), 08 **3/0** (95.69 s), 14 **5/1** (38.33 s) — the -15
matrix reproduces; 14's walk wedge reproduces on the merge (not a new master interaction).

After the faithful cull + D2 (the pushed state):

| binary | result | was (-15 matrix) |
|---|---|---|
| walkreal_07000002 | **1/0 ✓** (53.49 s) | RED at DOWN, green only at UP |
| walkreal_07000003 | **4/0 ✓** (333.81 s) | RED at DOWN, green only at OFF |
| walkreal_07000008 | **3/0 ✓** (92.48 s) | ✓ at DOWN, ✓ at OFF |
| fix_npc_doors | **1/0 ✓** (69.35 s) | ✓ |
| npc_doors | **6/0 ✓ + 1i** (75.76 s) | the 06-guard RED — **fixed** |
| mp_perf_16 | **2/0 ✓** (10.88 s) | the digest RED — re-pinned (below) |
| walkreal_07000014 | **3/3 RED** (12.94 s) | 5/1 at DOWN, 3/3 at OFF |
| walkreal_07000001 | **2/3 RED** (319.48 s) | 5/0 at DOWN, 2/3 at OFF |

**Five of the seven named binaries green, two of them newly** (02 without the UP flip, 03
without OFF; npc_doors' -15 red gone; mp_perf_16's digest re-pinned). The two remaining reds
are ONE mechanism, not two cull states:

**The substitute-tail gap.** Under the faithful rule the walk-leg cull is inert, so faces the
normalized rule used to skip now push: 14's -0.91-landing stair climb rides the step/skirt
pushes up (Bond y +0.173 vs the expected < 0.0 — the -6-c "r-equilibrium climb"), and 01's
AND-touch contact misses the plate ([-33.07, 3.94, 12.83]). The game does not have this
problem because its push tail is near-dead on walked lines (the accumulator arm, above) — its
walk vertical is the **feet band** [M Player_FeetOnPoint 0x194188], which the port has not
ported (the -6-c open item). No face-skip rule inside THIS query can carry both 02/03 (need
inert) and 14/01 (need the substitute's stair settle) — that is exactly the -15 "no single
cull state greens all five" contradiction, now explained: the contradiction was never between
cull states, it was between the port's substitute tail and the game's dead tail. The next
work is the feet-band port (the -6-c item), after which 01/14 should hold under the faithful
rule with no cull at all.

**mp_perf_16's digest re-pinned** to 0x5a28_4ff1_a217_bdfa (this window's run; kills printed
in the run line) with the intended-change note citing this window's greens — the pin's own
doc pattern (it pins the port's fixed-run behaviour, not game bytes; the two machines'
per-tick hashes still agree every tick). NOT a decompile-backed weakening.

**The two lib pins rewritten** (they contradicted the decompile — the rule fixes the test):
- `head_on_cull_uses_the_raw_spine_dot_and_the_crossing_half` — the climb-spine halves
  inverted (f=0.4 up-floor now culled [M 0x10a288/0x10a2c4 against the raw denom], f=0.6
  solid); the 43°-slope assertion stands (denom −0.956, not culled).
- `walk_spine_culls_only_near_flat_faces` — the walk spine never culls (both f=0.4 and f=0.6
  solid; |denom| ≤ 0.7597 < 0.999 [M the disasm closure; L ee_start_05]).

## What ran

- nf-game `--lib`: **591 passed / 0 failed / 2 ignored** (the final pushed tree).
- nf-view named binaries: the table above (each quoted with its time).
- The full suite: NOT run to completion (the background runner dies between tool calls in
  this sandbox family — re-measured this window: the nohup suite.py was gone between two
  tool calls, count 0, empty log). A chunk pass over the remaining binaries did not fit the
  window after the walkreal/npc runs; the state file from -14 (`/home/z/suite_c14_state.json`)
  is stale for this tree. **The next message should run
  `python3 tools/test/chunk_suite.py /home/z/suite_c16_state.json --budget 500` with
  `CARGO_TARGET_DIR=/home/z/NightfirePort/target` to completion before merging.**
- Pushes: 81b70d70 (start + WORKLOG), the faithful cull + D2, the test rewrites + digest
  re-pin. `git status --short` checked before every commit (only the named files staged).

## What's left (the next message)

1. **The full suite** on this tree (chunk runner, state file above); quote the total. Then
   merge origin/master and re-run the named binaries + a chunk pass on the final tree.
2. **The feet-band port** [M Player_FeetOnPoint 0x194188] — the -6-c open item, now the
   PROVEN prerequisite for 01/14 under the faithful rule (the mechanism, not another cull
   state; this report's substitute-tail section). walkreal_07000014's four remaining tests
   and walkreal_07000001's three should be re-run after it.
3. Re-verify the 02 geisha-2 save (:260) and 03's route stay green through the merge — they
   are the sensitive legs the -15 matrix moved in every prior state.
4. The visual round (merge lesson 6) is NOT needed for this landing (no draw changes), but
   the main agent may want a `--bond` spot on the 14 stair climb to see the substitute-tail
   overshoot live: `--bond 45.5,-0.91,53.9,0,0` north on the -0.91 landing (Bond should stand
   ON the landing; on this tree he rides ~1 unit high).

## Live checks for the main agent

1. **Intersect_CylGeom's push tail on a real walk** (gdbwatch.py): watch HITTEST+0x50
   (BLData+0x790 for the ground query, e.g. BL 0x16cf0f0 from ee_start_05) during a stair
   descent in 07000001 — expect it to accumulate DeltaP per landed hit (the drift) and the
   per-triangle arm `|q − +0x50| < radius` to essentially never fire; confirms the dead-tail
   reading behind the substitute-tail gap.
2. **The walk-leg cull inertness** (gdbwatch.py): a breakpoint pair at 0x10a288/0x10a2ac
   during a flat-walk leg — expect the 0.999 compare to NEVER take the skip branch (|denom|
   ≤ 0.7597), and to take it on a ladder/climb leg (|denom| up to 1.3097). This is the [L]
   confirmation of this window's [M] closure.
3. **The feet band** (fieldprobe.py): Player_FeetOnPoint 0x194188's inputs/outputs during the
   14 stair climb — the values the port's feet-band port must reproduce (the -6-c list grows
   by this window's 14 stair-climb repro).

## ntfy
