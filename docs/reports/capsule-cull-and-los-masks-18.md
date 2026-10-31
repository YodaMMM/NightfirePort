# capsule-cull-and-los-masks-18 — report

Task: continue `origin/s/capsule-cull-and-los-masks-17` (merge `origin/master` first; read its
report), bisect the 13 reds the merged full suite went to (the LOS masks on, the head-on cull,
the flags-99 skip, the turret muzzle/sphere change), find what still differs from the game, land
with every named binary green. Branch `s/capsule-cull-and-los-masks-18` =
origin/s/capsule-cull-and-los-masks-17 + merge origin/master (7d0091a0), clean; first checkpoint
pushed (0701a932).

## Status: the bisect is STARTED and three of the four suspected causes are EXCLUDED — not finished

This window ran out before a landing; the tree is left exactly at the merge state (0701a932,
clean — the bisect instrumentation was added, measured, and reverted). What the window
established, with the run receipts:

### The reds reproduce on this tree

`walkreal_07000014_live` (built alone, `--test-threads=1`): **3 passed, 3 failed** —
`the_east_duct_route_reaches_the_stairwell_slot`, `the_vent_drop_hands_bond_into_the_facility`,
`walk_the_infiltration_to_the_delta_exit` all fail at the SAME assert
(walkreal_07000014_live.rs:291/:443): `the east stairwell foot (-0.91 landing, z 51.5..54.25
[D probe14_corridor/probe14_stairs]) is reached, got [45.1893, 0.17301755, 53.1738]` — x/z are
inside the landing box (44.0..46.4 / 52.5..54.6), only `p[1] < 0.0` fails: Bond rests ~1.08 above
the -0.91 landing (the stairwell throat x 45..45.75 rim holds him — the test comment's "walkable
west edge x 44.5..44.75, the 45..45.75 middle is the stairwell's deep throat").

### The bisect matrix (gates added, measured, removed — the tree is back at HEAD)

Three env gates were added in one edit (default = the current behaviour, env var turns the rule
OFF — zero default impact, removed after measurement):

| run (the_east_duct_route_reaches_the_stairwell_slot) | result |
|---|---|
| baseline (the merged tree as pushed) | FAILED (y 0.173 > 0.0) |
| `NF_B_NO_CULL=1` (the raw-denom head-on cull skipped) | FAILED |
| `NF_B_NO_MASKS=1` (`line_of_sight` → the unmasked `segment_clear`) | FAILED |
| `NF_B_NO_TURRET=1` (the ext+0xd0 muzzle_offset block skipped) | FAILED |
| all three together | FAILED |

So the head-on cull, the per-caller LOS masks and the turret muzzle/sphere change are ALL
EXCLUDED as the east_duct red-maker. (The Dist2Tri flags-99 skip is not testable as a delta:
master carries it env-gated `NF_CAPS_D2` default-off, and the -17 tree DELETED those three lines
— world.rs diff shows master's `if *CAPS_D2...&& dot(sub(p, t[0]), n) < 0.0 { return false; }`
replaced by a comment that claims "LANDED NOW" with no code following it. The rule is absent
from both trees at default settings; the comment is wrong about the code.)

### The remaining candidates (the next window's first work)

The master drift since -17's own merge (fd24fbf4) was then enumerated — the collision spine
(world.rs/player.rs) did NOT move on master; the only functional drifts are on THIS branch:
- the un-gated remainder of the world.rs capsule delta (231 lines vs master): the walkable-gate
  sign discipline (`n[1] > 0.7` + `q[1] < b[1]`, up-facing faces only — the level-14 hood
  fix), the pick bit's dot2 pass-global (`pick_ref_y < b_pre_y` applied once per query), and
  the pick-contact ref changes. **The next window should extend the gate set to these and
  re-run the matrix above.**
- `tests/common/walker_b2.rs` (master, post-fd24f): the stuck-recovery jumps now check 2.2 m
  headroom (the -10 stairwell-throat fix). EXCLUDED for east_duct by measurement: reverting
  walker_b2.rs to fd24fbf4 and rebuilding (BUILD=0) left east_duct FAILED (2.02 s run).
- `crates/nf-view/src/scene.rs` (+33: rigid bodies ride posed slots) and `door.rs` (NF_DOORTRACE
  lines) and `tests/common/walker.rs` (NF_DOOR_TRACE lines): draw/trace-only, excluded by
  inspection.
- The 14 test file itself: byte-identical since fd24fbf4.

### Why the window ran out

- The sandbox's tool-output path silently eats the bytes `[m` (an ANSI-strip bug: `maps[m]`
  displays as `maps]`, `[move_v` as `ove_v`). ~50 minutes went to chasing phantom source
  corruption that the hex dump disproved — the pushed -17 tree was never broken (`git show
  origin/s/capsule-cull-and-los-masks-17` + raw.githubusercontent agree with the working tree;
  `cargo build -p nf-game` finishes clean). Anyone reading earlier reports' `ove_v`/
  `maps]` strings: the real text has `[m` in those spots.
- Background runners die between tool calls here (re-measured: the suite.py run died with an
  empty suite.out twice), so everything ran foreground; the full 850-binary build (~10 min per
  chunk) and 15 test binaries were the whole build budget.

### What is NOT done

1. The remaining world.rs delta gates + the full 13-red matrix (only east_duct was bisected).
2. The fixes themselves — nothing is fixed yet; no test was changed.
3. The FULL suite (the fast runner cannot survive this sandbox; the named 15 binaries built,
   only walkreal_07000014_live ran: 3/3 above; the other 14 binaries were built, not run).
4. No code change landed; the two capsule rules -17 verified are untouched on the pushed tree.

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
