## Your task: `mission-stats-carry`, the pause SCORE counters must survive a mission's level change

Symptom: the pause SCORE tab counters reset when The Exchange moves from 07000005 to 07000006.

1. Find in the decompile what the game keeps across a mission's level change: PlrMissionStats (docs/fields/player-stats.md). Who zeroes it (mission start vs every level load: `ResetMap_Load`, `Player_Init`, the `PlrStat_*` init) [M], and what does the SCORE tab's `C_GCPAUSE_Handler` 0x204408:447-498 read?
2. Make the port carry exactly what the game carries across the level change, and reset exactly where the game resets (a new mission, a restart).
3. Real-path test: on 07000005, earn a Dispatched and a Bond Move, cross into 07000006 through the real level exit, and the counters still hold. Sister: restarting the mission zeroes them, if the game does [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-stats-carry`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-24T12:14Z
