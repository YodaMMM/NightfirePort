# capsule-cull-and-los-masks-13 — report

Task: the -12+master merge turns 4 tests red reproducibly (green on master):
walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit,
walkreal_07000003_live::walk_the_full_route_and_complete_every_objective,
fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route,
walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit. Split the two family changes
(the head-on capsule cull; the LOS masks/masked_scan), find which moves each red, fix the port
where wrong. Branch `s/capsule-cull-and-los-masks-13` = origin/master (21fbeed3, which contains
8dc61a92) + merge origin/s/capsule-cull-and-los-masks-12 (20270f58).

## The bisect (NF_NO_CULL / NF_NO_MASKS gates, landed as bisect tooling)

Reproduced on the merged tree first: 07000002 FAILED at :260 in 34.5 s; 07000003 FAILED at :215
("servant 0 left WAIT_SWITCH/HOSTAGE, got 0x1") in 153.0 s; fix_npc_doors exceeds 280 s solo
(not finished this window); walkreal_07000014's named test is `#[ignore]`d on this tree (the
level-14-skip-handback ignore predates the family) — it cannot go red as-is.

| config | 07000002 (34-58 s) | 07000003 (73-153 s) |
|---|---|---|
| merged tree (cull + masks) | FAILED | FAILED |
| NF_NO_CULL=1 | **ok** (52.1 s) | **ok** (148.0 s) |
| NF_NO_MASKS=1 | **ok** (49.9 s) | FAILED (92.7 s) |

→ **The head-on cull is the common mover; the masks contribute only to 07000002** (both gates
green it; either alone suffices there).

## The root cause: the cull's direction test was inverted [M disasm]

`Intersect_CylGeom` 0x109da0's setup, from the ELF disassembly (work/elf/ACTION.ELF):
- `0x109f60 lwc2 $1, 0x20(v1)` — HITTEST+0x20 (A, the spine TOP) → vf1;
- `0x109f68 lwc2 $2, 0x30(v1)` — HITTEST+0x30 (B, the BOTTOM) → vf2;
- **`0x109f70` VSUB vf1 = vf1 − vf2 = A − B**;
- `0x109f74 jal 0x1e3000` — `Vec_Normalise`.

The game normalizes **A − B (TOP − BOTTOM — the cull dot points UP)**; the port normalized
B − A (DOWN) and applied the decompile's own skip conditions (:321-322: `(0.999 < n·d̂ ∧ 0.5 <
f) ∨ (n·d̂ < −0.999 ∧ f < 0.5)`) to the DOWN dot — which **inverts the f-halves for every face
orientation**: a floor crossed in the spine's top half was skipped (the "jump-through") where
the game skips the BOTTOM half, and vice versa. (The old "raw-dot is unreachable — |d| =
0.7597" [L ee_start_05 BL 0x16cf0f0] argument stands; only the direction was wrong.)

**The fix landed**: `nd_up = −(n·(B−A))/|B−A|` with the decompile's conditions unchanged
(crates/nf-game/src/world.rs, capsule_tri_contact's cull block, cites 0x109f60-f74). The two
cull lib tests (`head_on_cull_uses_the_raw_spine_dot_and_the_crossing_half`,
`walk_spine_culls_only_near_flat_faces`) pinned the inverted halves — **re-derived with the
address** (their f=0.4/f=0.6 expectations swapped; the 43°-slope cases unchanged — they never
crossed the 0.999 cone).

With the game's UP dot the semantic reads: an up-facing floor crossed in the spine's **bottom**
half (f > 0.5) is SKIPPED — the game's underfoot support comes from the feet ray
(Player_FeetOnPoint 0x194188), not this query — and the same floor crossed in the **top** half
is solid.

## What ran, on the fixed tree

- `nf-game --lib`: **591 passed / 0 failed** (0.08 s) — the re-derived cull tests + the masks
  tests all hold.
- `walkreal_07000001_live`: **5 passed / 0 failed** (85.7 s) — the stairwell-descent class the
  old DOWN reading was originally justified by stays green on the game's direction.
- `walkreal_07000002_live`: **1 passed / 0 failed** (58.3 s) — **the task's red is GREEN**.
- `walkreal_07000003_live walk_the_full_route`: still **FAILED** (72.9 s, was 153.0 s — the
  walk now gets much further) at :215, "servant 0 left WAIT_SWITCH/HOSTAGE, got 0x1": the
  servant's captor still survives the harness shoot(). A second-stage issue (the masks'
  contribution to the shot path, or the harness shot's pose) — NOT bisected further this
  window.
- fix_npc_doors / walkreal_07000014: unverified on the fixed tree (time).
- **FULL suite: NOT run this window.** The sandbox kills background runners between tool calls
  (the -12 report's 9th measurement; my setsid runner died inside 8 s), and the chunked resume
  runner (`tools/test/chunk_suite.py`) needs ~40-60 min the window didn't have after setup +
  the cold build + the bisect matrix (6 gated runs, ~9 min each). The next message should run
  it on this tree and quote the total line.

## What's left

1. FULL suite (chunked) on this tree — the flip touches every walk; the spot checks above are
   green but the 896-binary verdict is missing.
2. walkreal_07000003's remaining red (the servant-0 captor kill) — bisect with the gates
   whether the masks alone now move it (on the fixed cull).
3. fix_npc_doors + walkreal_07000014 on the fixed tree (the latter needs its ignore reviewed).
4. The gates NF_NO_CULL / NF_NO_MASKS are bisect tooling in world.rs — remove once the suite
   verdict lands.

## Live checks for the main agent

1. Fresh boot, any interior level: gdbwatch HITTEST+0x20/+0x30 during a walk onto a step —
   confirm +0x20 stays the upper end (A) and the cull dot is n·(A−B) [M 0x109f70], and that a
   step crossed in the spine's bottom half produces NO push from Intersect_CylGeom (the feet
   ray carries the support).
2. The servant-captor shot on 07000003 (fresh boot): does the game's bullet reach the captor
   from the walk's settle pose — i.e. is the remaining red a port masks issue or a harness shot
   placement issue?

## Usage

not visible to me — this session exposes no token, credit, cost or context-window figures in
any environment output I can read.
