## Your task: `vision-bit-unported-states`, the drone states the vision-bit census found missing

`level-13-guards-start-alerted-3` (read its report and `crates/nf-game/tests/vision_bit_census.rs`) pinned all 137 game states that write the drone vision bit `+0x4F8 |= 0x10000` [M, NDrone2_StateFuncs 0x29b300], and its `NOT_IMPLEMENTED` table lists the 14 writer states the port has no arm for. Port each of those states from the decompile (ENTER/UPDATE/EXIT as the game runs them, including the vision bit), wire them where the game dispatches them, and move each row from NOT_IMPLEMENTED into the census's required table. Test each new state on a level that uses it [D placements/scripts] on real input; the AI/escort/walk family stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/vision-bit-unported-states`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T02:11Z
