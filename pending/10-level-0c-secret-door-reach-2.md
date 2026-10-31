## Your task: `level-0c-secret-door-reach-2`, 0c: walk in through the corridor's north end

Continue `origin/s/level-0c-secret-door-reach` (merge `origin/master` first; read its report in `blocked/10-level-0c-secret-door-reach.md`). It found the route [D]: the plates' corridor floor (0.90) runs north past the vent and joins the warehouse ground (0.69/0.89) — yard, warehouse ground and corridor flood as ONE component — so the plates #558/#559 are reachable on foot from the north end. Write the walk legs on real input (yard -> warehouse ground -> the corridor north end -> south to the plates), open the secret door #544 through the plates (body touch #559 + E #558 -> AND #560 -> ch 8), walk out through it, and delete the `[plate2 + 0.6]` placement in `walkreal_0700000c_live`. Run walkreal_0700000c_live and interactables_0700000c_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-secret-door-reach-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

