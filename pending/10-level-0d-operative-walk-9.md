## Your task: `level-0d-operative-walk-9`, 0700000d's Operative arrival-to-C5-door real-damage walk goes green

Branch from current origin/master; if origin/s/level-0d-operative-walk-8 is not on master, merge it in first. Read docs/reports/level-0d-operative-walk-8.md there and the ignore message on walk_arrival_to_the_c5_door_real_damage. Lever 1 is answered; take the crossing-sweep lever the report and ignore message name next. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-operative-walk-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

