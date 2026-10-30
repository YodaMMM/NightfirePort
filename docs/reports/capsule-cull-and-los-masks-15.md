# capsule-cull-and-los-masks-15 — report

Task: root-cause the second cull divergence (the -14 bisect: NF_NO_CULL=1 greens the
walkreal_07000003 captor-kill red, NF_NO_MASKS=1 does not) against [M Intersect_CylGeom
0x109da0, Collide_Intersect 0x1ebaa8]; bisect walkreal_07000008's fusebox red; capture and fix
the mp_perf_16_live and npc_doors_live reds; REMOVE the NF_NO_CULL / NF_NO_MASKS gates. Land
only with every touched binary green and the walkreal binaries for 02, 03, 08, 14 green.
Branch `s/capsule-cull-and-los-masks-15` = origin/s/capsule-cull-and-los-masks-14 + merge
origin/master (64abcb78), clean.

## The headline: the five walks require contradictory cull states

Every state below is [L] measured this window on the -15 tree (real input, the walkreal
binaries), except where marked from the -13/-14 reports:

| contact cull | clip cull | walkreal_01 | 02 | 03 | 08 | 14 |
|---|---|---|---|---|---|---|
| DOWN (master's default) | DOWN | **5/0 ✓** | RED :260 (geisha 2 saved) | RED 154 s | **3/0 ✓** | **5/1** (the walk wedges at "the corridor south to the leg-2 spot", 6000-tick cap) |
| UP (the -13 fix) | DOWN | 5/0 ✓ | **1/0 ✓** | RED :215 (servant 1, state 0xa) | RED (the -14 datum) | 2/4 (the pit-band approach wedges at x 42.3) |
| OFF | DOWN | RED 2/3 (the stairwell class breaks) | ? | **ok 148 s** | **3/0 ✓** | 3/3 |

02 needs UP, 14 needs DOWN (and its remaining walk wedge is not cull-moved — it survives
NO_CULL), 03 and 08 need OFF, 01 accepts any. **No single cull state greens all five walks.**
The port's push/clip split cannot be right in one state — that structural split is the true
second divergence, and the next window must resolve which arm the game's ONE cylinder query
really culls (or port the game's single query faithfully).

The landing on this branch: **default DOWN = master's state** (the smallest delta from the
merged baseline, and it flips walkreal_07000008 GREEN — a new win: the -14 tree had it red
under UP), with the direction switchable (NF_CULL_UP) and the off switch (NF_NO_CULL) kept as
the documented bisect tools. **The NF_NO_MASKS gates ARE removed** — the per-caller drop
filters in `masked_scan`/`line_of_sight_hit` are now unconditional (world.rs), which is
master's own behaviour (the gates were opt-IN disables; removing them changes nothing by
default and deletes the gate code as the task demands).

## What was pinned this window [M disasm, work/elf/ACTION.ELF via tools/crosscheck/mipsdis.py]

1. **The cull compare site**: 0x10a278-0x10a2e8 — `c.4.s` (OLT) against 0.999f
   (0x3F7FBE77, loaded via `lui at,3f7f; ori at,be77` — NOT a literal-pool float, which is why
   a byte-grep for 0x3F7BE76C missed it), the two skip arms exactly as the port has them
   (`0.999 < dot ∧ 0.5 < f` ∨ `dot < −0.999 ∧ f < 0.5`), branching to the tri-loop tail
   0x10a7b8.
2. **The f clamp is the CONSTANT pair**: f20 = 1.0 (mtc1 at 0x10a028) and f23 = 0.0
   (0x10a01c), and a grep of the whole function's disassembly finds no other write to either
   register — the "running interval" hypothesis is refuted; the game's fraction clamp is
   static [0, 1], identical to the port's. The clamp shape [M disasm 0x10a238-0x10a264]:
   f = 0 when frac < f23(0), f = 1.0 when frac > f20(1.0), else f = frac.
3. **The -13 report's setup reading was backwards**: 0x109f60/0x109f68 are `sqc2` (STORES of
   vf1/vf2 INTO HITTEST+0x20/+0x30 — the query's A/B being written), then the VSUB and
   Vec_Normalise(sp+48) — the normalized spine lands on the stack, and the per-triangle dot
   loads it from there. The dot's exact input vector (sp+48's spine vs another vector) is the
   one link the byte-level analysis did not close; the decompile's second dot uses
   param_1[2] = HITTEST+0x20 (A) as the fraction's origin [M decompile :295-299].
4. The fraction's near/far gates [M disasm 0x10a3a4-0x10a3dc]: the second crossing's fraction
   is dropped when beyond f20 or before f23 — the same constant pair, so no additional
   divergence there.

## The other reds, captured

- **mp_perf_16_live::sixteen_participants_fixed_run_digest_is_pinned** — "the optimisations
  changed the simulated result" (mp_perf_16_live.rs:176, a fixed-run digest mismatch). Not
  bisected this window.
- **npc_doors_live::the_06_guard_chasing_bond_opens_the_closed_door** — "the door opened but
  the chasing guard never crossed to Bond's side" (npc_doors_live.rs:638). Not bisected.
- **walkreal_07000014** (master's newly un-ignored walk + 3 pin tests): at master's DOWN
  default 5/1 — the four pit-band reds of the UP state drop to one walk wedge, and that wedge
  survives NF_NO_CULL (so it is NOT the contact cull); it is either a master-merge interaction
  or a masks-side effect. On master's own tree the level-14 task reported the binary green —
  the delta needs a master-tree run to attribute.

## What ran

- `nf-game --lib`: **591 passed / 0 failed** (the final tree).
- walkreal binaries on the final (DOWN-default) tree: 01 **5/0**, 02 **1 red**, 03 **1 red**,
  08 **3/0**, 14 **5/1** — each quoted above with its per-test state; the bisect runs behind
  them: 03 + NF_NO_CULL ok 148.17 s, 08 + NF_NO_CULL ok 96.24 s (measured earlier on the
  UP tree), 14 + NF_NO_CULL 3/3, 14 + NF_CULL_UP 2/4.
- The full suite: NOT run to completion this window (the background runner dies between tool
  calls in this sandbox family — the -12 report's 9th measurement; the chunked runner needs
  the remaining minutes and the state file below carries what was recorded). Next message:
  `python3 tools/test/chunk_suite.py /home/z/suite_c15_state.json` with
  `CARGO_TARGET_DIR=/home/z/NightfirePort/target`.

## What's left (the next message)

1. **Resolve the structural split**: port the game's ONE cylinder query faithfully (or prove
   which arm the cull really serves — the push-out's radial arm reads the same
   Intersect_CylGeom on the game side, yet every faithful-looking port of its cull to the
   port's push arm breaks two walks while fixing two others). The bisect tools to do it live:
   `NF_CULL_UP=1` (the contact dot UP), `NF_NO_CULL=1` (the contact cull off),
   `NF_NO_MASKS` is gone (the masks are unconditional).
2. **walkreal_07000014's walk wedge** at the DOWN default (not cull-moved) — attribute it
   against a master-tree run of the same binary.
3. **mp_perf_16's digest red** and **npc_doors' 06-guard red** — texts captured above, fixes
   not attempted this window.
4. **The full suite** on the final tree merged with origin/master; quote the total line.
5. Only then remove the two remaining bisect switches (NF_CULL_UP / NF_NO_CULL) and land.

## ntfy

Sent successfully: curl, HTTP 200, id ZNBRkLgsug3I (2026-10-30 ~12:28 UTC), message
"capsule-cull-and-los-masks-15: matrix landed — no single cull state greens all five walks
(02 UP, 14 DOWN, 03/08 OFF, 01 any); landing = master's DOWN default (08 flips green), masks
gates removed, disasm pinned; report on branch".

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
