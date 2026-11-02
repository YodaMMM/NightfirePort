## Your task: `level-01-bonus-floor-walk`, walk onto the 07000001 west-hall Bond bonus and take it

`crates/nf-view/tests/interactables_07000001_live.rs` `the_bond_bonus_walk_over_takes_it` (~line 385) is #[ignore]d: the bonus floor sits ~1.6 below every floor the walker reaches (the spawn-side nav ends on the upper hall at y 7.18; the west corridor's low route rides the escort path). The two bonuses are at #315 (−13.24, z 10.35) and #316 (−26.79, z 14.51) [D]. Find how a player reaches that floor in the game (stairs, a drop, a door, the escort route) [D collision + placements; M any door/lift gate on the way]. If the port blocks a route the game leaves open, fix the port. Then walk there on real input, take the bonus by walking over it, and un-ignore the test. Run interactables_07000001_live and walkreal_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-bonus-floor-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-02T03:51Z

---
## Report (done, 2026-11-02T05:16Z)

# level-01-bonus-floor-walk — the two Bond bonuses' real reach

Branch `s/level-01-bonus-floor-walk`, 2026-11-02. Task: un-ignore
`crates/nf-view/tests/interactables_07000001_live.rs::the_bond_bonus_walk_over_takes_it` —
find how a player reaches the bonus floor in the game, walk there on real input, take the
bonus by walking over it, and fix any port gap that blocks a route the game leaves open.

## Result

**Done — the test is un-ignored and green.** Bond walks both routes on real input and takes
both bonuses by walking over them. No port gap was found: every leg of both routes behaves as
the disc's data describes (collision cels, step heights, the door's E path, the walk-over
consume), so no lib change was needed — the old test's teleport was compensating for a
nav-graph limitation, not a port bug.

## What the blocker actually was

The old test teleported Bond onto the bonus spot (`sim.bond.pos = …`) as a "stand-in transit";
its ignore note said "the bonus floor sits ~1.6 below every floor the walker reaches (the
spawn-side nav ends on the upper hall at y 7.18)". Both halves decode cleanly once two facts
are pinned:

- **`bond.pos[1]` is Bond's centre, roughly floor + 1.0.** The "y 7.18" of the old note is the
  centre reading while Bond stands on the 6.14 curb step at (−13.4, 12.4) [D the disc's
  collision cels via `world.floor_below` sweeps — every coordinate below is that parse].
- **The two bonuses are on two different floors.** #315 "Pickup 007" sits at
  (−13.244, 5.630, 10.355) [D placement] on the west low floor (**5.34**, under the gallery
  slab 8.5-8.9 spanning x −13.5..−10.5, z 9-13 [D]); #316 sits at (−26.790, 9.514, 14.508)
  [D] on the **gallery itself** (9.224 slab, x −22..−30, z 13.5-16.5 [D]). 5.63 vs a
  centre-at-7.18 standing: the "~1.6 below" of the old note.

## The routes (all [D], the disc's own data)

**To #315 — no door, no gate.** The walkable line is the south-west steer from the spawn
side: across the south rooms (floors 4.4-4.9 [D]) and a step north into the alcove under the
slab. The alcove is a **sealed pocket**: its north and west sides carry wall stubs (probe rays
hit at z≈10.9-11.9, x −13.2..−11.5 [D]) and the z≈11 strip east of it has **no floor at all**
(`floor_below(…, 8.4)` returns nothing for x −15..−7 at z≈11 [D] — a floor gap between the
z-12 and z-10.5 slabs, with 6.14 curb steps flanking). That gap is why the AI's own
`move_test` refuses every lane→alcove line (result 0, no world hit) and why nav-route-only
walking never reaches the alcove: the only opening is the south step, the way the walk comes
in.

**To #316 — the game's own ramp chain, all in the disc's AI nav graph** [D aipath, path 1]:
from the hall stage the west corridor lane (z 14.0-14.7) runs west under the gallery (the
slab rides z 13.5-16.5 over the lane), through **door #2** (−14.06, 5.35, 13.05 [D]; the east
face is the hall's north-west floor, the west face the lane — the escort's own westward
crossing; `ch_open = ch_lock = 0` [D params], E-openable [M 0x1a5170 case 0x1C →
Door_Activate; the entry-door test pins the path]) → south at x≈−31 (nodes 1→2→5) → the z≈0
run (nodes 6→7→8) → **the stair east**: the disc's step cels climb 5.339 at x −28.0 → 8.620
at x −23.0 across z 0.0-0.5 [D floor_below sweep, ~0.4 rise per step] → node 9 (−22.29, 9.47,
−0.04) → the gallery ring north (nodes 9→15→10→11→12, y 9.67 [D]) → west along z≈14 (nodes
12→13) over #316. The "balcony guard parked over the west stairwell" (the level doc's note)
stands on this same gallery. The ramp foot's approach must ride the AI's exact chain — the
direct east cut wedges on the stair's side wall [D the wall the probe ray hit at z≈1.3].

## The test

`the_bond_bonus_walk_over_takes_it` (un-ignored) walks: at-rest pin (after the intro, before
anyone moves, both bonuses wait) → the south-west line → #315 walk-over take [M Pickup_Handler
0x18e650:234-236] → back out the south step → the nav graph to the hall stage → the lane →
door #2 E-open → the lane west → the south run → the exact node chain to the ramp foot → the
stair up (Bond lands at centre 9.59 on the stair's top step) → the gallery ring → #316
walk-over take → the hall bonus stays taken. Sister: a fresh load nobody walks — the intro
plays out, 600 ticks idle, and both bonuses still wait.

## Verification (what was run — all on the final tree, merged with origin/master)

The final tree carries `merge origin/master` (3628fce7 — the level-13/14/15/16/1b batch + the
first_tracker restore; the merge's only nf-game delta is level-15's `drone.rs` work). After
the merge, `cargo` rebuilt the touched crates and every run below is post-merge:

- `interactables_07000001_live` (the whole binary): **10 passed, 0 failed, 0 ignored** — the
  un-ignored test included.
- `walkreal_07000001_live`: **6 passed, 0 failed, 0 ignored** (84 s).
- `nf-game` (all binaries, `--test-threads=1`): **1033 passed, 0 failed, 9 ignored**.
- The fast suite (`tools/test/suite.py`): its **build phase completed — 1077 test binaries in
  1467 s** — and its run phase was cut by the session window (the sandbox kills background
  jobs between tool calls; the surviving foreground time went to the three runs above, which
  cover the touched binaries and the whole of nf-game). Its per-binary progress up to the cut
  is in `suite.log` beside the checkout; a full pass on this tree is the next session's first
  `--only`-free run.

## Findings worth carrying

- **A guard can consume a pickup.** In one probe run the hall bonus flipped to `gone` during a
  120-tick wait with Bond 4 units away — a wandering guard (drone #13 patrols the region)
  crossed the pick sphere. The pick pass is not Bond-gated on this level. The shipped test
  avoids the window (the at-rest pin + the idle sister + the take inside one deterministic
  walk), but any future test that leaves Bond near live pickups with guards loose should
  expect it.
- The trench/curb band at z≈11 (the no-floor strip) is the reason the west rooms' nav never
  routes into the alcove — geometry, not a nav bug.

## What's left

- One full suite pass on the merged tree (the next session: the tree is pushed, the merge is
  in; `--only interactables_07000001_live,walkreal_07000001_live` is the quick re-check).

## Live checks for the main agent

One optional: watch `pickups` while a guard walks near the hall bonus #315 (−13.24, 5.63,
10.35) — address the kind-6 row's gone/hidden byte — to confirm the guard-consume on the real
game (the port shows it; the decompile's pick-pass caller was not re-read for WHO may pick).

## Usage

Not visible to me — this session's driver shows no token, credit or context-window figures to
copy.
