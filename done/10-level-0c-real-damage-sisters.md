## Your task: `level-0c-real-damage-sisters`, 0700000c's real-damage and Agent walks are green with the game's guard sight

Branch from origin/s/guard1-rounds-0c-repin, merge current origin/master, and read its docs/reports/guard1-rounds-0c-repin.md plus the ignore messages on the two ignored walkreal_0700000c_live sisters (the real-damage and Agent walks). real-health-agent-0c-0d-3 may also be working on the Agent walk's 0c legs: check origin/s/real-health-agent-0c-0d-3 first and build on it rather than repeating it. Take each ignore message's levers in order, judging each over several real runs. Helpers stay gated to 0700000c and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000c_live binary with its eastroute and barrelrow sisters, and interactables_0700000c_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-real-damage-sisters`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T13:09Z

---
## Report (done, 2026-11-11T14:53Z)

