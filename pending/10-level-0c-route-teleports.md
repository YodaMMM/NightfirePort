## Your task: `level-0c-route-teleports`, 0c: the three walk teleports

`walkreal_0700000c_live`'s full walk teleports Bond three times: out of a shaft he can drop into but not climb out of, onto a mezzanine reached (as far as the tests show) only by the Q-Claw from the warehouse, and onto an east deck no collision connects to. For each, find the game's route [D collision and placements: ladders, grapple points (Q-Claw targets), lifts, doors; M the classes the route uses], fix any port gap with the address cited and walk it on real input; where the route is a grapple, drive the real Q-Claw. Delete each teleport you replace; keep the rest with a sister assert and name the live check. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live, interactables_0700000c_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-route-teleports`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

