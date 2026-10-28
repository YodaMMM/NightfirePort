## Your task: `level-14-vent-drop-2`, implement Countdown Part 1's vent drop

Start from `origin/s/level-14-vent-drop` (merge origin/master) and read `docs/reports/level-14-vent-drop.md`: the previous session decoded the whole chain (touch 0xd9 -> OR 0x7e -> the SP switch; the case-6 capture -> Script_Free's Player_Enable) and wrote "The fix" section, then was cut off before implementing it. Implement that fix, scoped as the report says, then walk `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` legs 2..6 to the exit on real input; un-ignore on green, else back to `#[ignore]` with the exact blocker. Run the crates and tests the report lists. Commit and push progress early.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-vent-drop-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T01:50Z

---
## Report (done, 2026-10-28T03:23Z)

Closed by the driver: the report is on `s/level-14-vent-drop-2` (docs/reports/level-14-vent-drop-2.md); the worker didn't run finish.sh.
