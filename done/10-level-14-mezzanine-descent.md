## Your task: `level-14-mezzanine-descent`, Countdown Part 1: the walk from the vent scene down into the facility

Read the level-14-skip-handback report (on master). The vent cut-scene 0x06000833 now hands Bond back where he stood, as the game does [M 0x1d94c0 case 6; L PCSX2 fresh 07000014: Bond triggers touch #301 at (40.7, 3.1, 39.0) and stands at (40.68, 2.76, 38.99) after the scene, audits/live-14-ventscene in the main agent's notes]. `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` is ignored because its old route rode the removed teleport and the mezzanine descent wedges at (32.06, 2.70, 39.4) (the wall at x 31.5, z 38.4..40.8, and the floor edge south of z 38.4). Find the route a player takes from that spot into the facility [D the collision and placements around the mezzanine: stairs, ladders, drops, doors, vents; M any use/jump path], fix any port collision gap, walk it on real input, and un-ignore the test with the rest of the walk green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-mezzanine-descent`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T21:24Z

---
## Report (done, 2026-10-29T23:59Z)

Closed by the driver: the report is on `s/level-14-mezzanine-descent` (docs/reports/level-14-mezzanine-descent.md); the worker didn't run finish.sh.
