## Your task: `level-4a-rappel-vanish`, rappelling guards vanish when shot

In real play on the 0700004a lift ride, "rappel guards disappear after shot": a guard on a rope disappears instead of dying/falling when hit. Find the 4a abseil guards [D], drive the real ride with real input, shoot one, and log its state/anim/visibility per tick. Compare with the game [M — NDrone2_DSTATE_AbseilSlide 0x169730 / AbseilHang 0x169940 damage arms, the death on a rope (the fall, the corpse), `abseil_death_live` on 11]. Fix the port so a shot rope guard does what the game's does; a real-input test, and headless renders named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-rappel-vanish`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-25T20:10Z

---
## Report (done, 2026-10-25T21:00Z)

Closed by the driver: the report is on `s/level-4a-rappel-vanish` (docs/reports/level-4a-rappel-vanish.md); the worker didn't run finish.sh.
