## Your task: `level-0b-roof-sprint-5`, the 0700000b roof sprint can be walked on real input

Branch from origin/s/level-0b-roof-sprint-4, merge current origin/master, and read its docs/reports/level-0b-roof-sprint-4.md plus the ignore messages. The rocket deck drop kills Bond because of the port's blast range, which explode-hurt-radius owns, so do not touch the blast here. Take the other option it gives: take #4 with the shotgun from inside 9 u (the range the fire-on-move proved, #7 landed at d 9.2) on a route that closes the gap before the turn. Then the sisters, then the whole 0700000b binary. Helpers stay gated to 0700000b and the difficulty. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run walkreal_0700000b_live and interactables_0700000b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-roof-sprint-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-10T00:08Z
