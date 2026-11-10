## Your task: `level-0b-roof-sprint-9`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/level-0b-roof-sprint-8; if origin/s/los-mask-repairs has moved past it, merge that too; then merge current origin/master. Read docs/reports/level-0b-roof-sprint-8.md and the 00 Agent ignore message in walkreal_0700000b_live. Sprint-8 reproduced the crosshair rocket (the #6/#7 pair dies to one round, three runs) and mapped the traps (#4 steals the lock, the stand bleed). Take its next levers in order. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T11:03Z

---
## Report (done, 2026-11-10T12:38Z)

