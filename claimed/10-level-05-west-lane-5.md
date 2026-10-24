## Your task: `level-05-west-lane-5`, 07000005: the slab crossing, then the notch grab, the ride and the south legs

On master the west lane and the crawl #299 legs walk green, and the creepwall half-length is the world-sphere radius x scale [M Control_BuildWorldSph 0x133070]. Two tests stay ignored: `the_cable_legs_the_crawl_notch_and_ride` and `the_notch_holds_the_walk_and_the_grab_needs_its_zone`.

The slab crossing has two measured blockers:
- [D] The slab's east tip is fragmented: z 52 is void at x <= 23.75; (24.0, 54) reads 25.9 between the 33.8 step at (24.0, 52) and the 33.3 strip at (24.0, 55); the walkable strip runs z 54-55 at x 2-14. Map x 14..26, z 44..56 at 0.25 units and find the game's link from the east steps (x 24-25.75, z 49-52) to the strip.
- [L port] On the slab top the walker's turn+forward input leaves Bond ungrounded 1.34 above the floor at (25.42, 49.55) and (19.15, 54.55) (floor_below 33.34), while a plain forward push grounds him there. The previous round blamed the breakables one-contact change (f56c1829c), which touches only bullets and breakables: bisect it (run the walk at f56c1829c^ and f56c1829c) before believing it, and find the real cause in the player's grounding.

Then the notch grab at icon #310 (-23.80, 32.74, 12.33), the ride, the south ledge and the window-ledge crawl #35 (the round-3 legs are in git history at 96cebefd1), real input only, and un-ignore both tests.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-24T00:52Z
