## Your task: `level-15-agent-rim-shot-2`, 07000015 can be walked on Agent with health to spare

Branch from origin/s/level-15-agent-rim-shot, merge current origin/master, and read its docs/reports/level-15-agent-rim-shot.md, section 5. The Agent walk is green and repeatable, but it reaches the exit with 2.0 health, one hit from death, so it is green only by a thin margin. Take its margin levers: bank the plank pre-shoot on #17, and the great-hall partner kill, so Bond crosses with 25 or more health as the doors-2 tree did. Re-pin the ledger with the real margin. The 00 Agent sister is not this task. Helpers stay gated to 07000015 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-15-agent-rim-shot-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-10T11:57Z
