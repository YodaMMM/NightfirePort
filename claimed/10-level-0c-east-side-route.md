## Your task: `level-0c-east-side-route`, the game's own route to 0c's east band

The 0c whole walk still teleports across the gate band (54.5, 11.0, -207.6) -> (68.0, 11.4, -207.9). A fresh-boot live check settles the straight crossing: walking east off the band in the real game, Bond DROPS into the pit (root y 11.37 -> 7.21 by x 61.5) and stops at x 63.4-63.5 against the pit's east wall, from two starts [L 2026-10-30] — exactly the port's disc collision (pit floor plc28 6.16..7.19, sliver to 64.05, +3.2 face; level-0c-gate-band-crossing report). So the game's route to the east side is elsewhere. Candidates from that report's [D] map: the turbine staircases (turb2 rise 2.97->11.17 at x 49.29..58.73 and turb4 0.16->11.20 at x 78.14..87.59, both z -208.96..-211.67), a lower floor linking them, the catwalk 15.5 (x 40.7..79.6) and its access, or the mission's intended order reaching the east side from another area. Find the walkable route [D coll-scan / placements / nav], walk it on real input, and replace the teleport. The walk must stay green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-east-side-route`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T17:13Z
