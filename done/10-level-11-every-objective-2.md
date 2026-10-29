## Your task: `level-11-every-objective-2`, land the 07000011 mission-row walk

`origin/s/level-11-every-objective` is green but cannot merge: it asserts `!ch 0x48` as game truth ("no writer exists") - the rule is that an assertion may not rest on a missing writer (the port may simply lack it). Merge `origin/master`, turn that assert into a `// [S] unverified` log kept out of pass/fail, keep every other assert, and hunt the 0x48 writer [D every placement/script/SP channel write on 07000011 and the shared objective machinery; M Mission_MonitorObjectives 0x1827d8]: if found, port it and assert the game's behaviour; if not, say so in the report with what was searched. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T01:30Z

---
## Report (done, 2026-10-29T03:03Z)

