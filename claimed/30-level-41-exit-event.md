## Your task: `level-41-exit-event`, 07000041 ends into the space station the way the game does

`crates/nf-game/src/drone.rs` (~line 15793, "The 07000041 wrapper end (documented stand-in...)") ends 07000041 with a port timer: once ch 1 is on it counts 241 ticks (the chain's longest script 0x06000027) and jumps straight to 0x0700001b. The comment records that 07000041.bin has no 0xE8 LOAD_LEVEL placement and no ch-100 writer, so the game's carrier must be a script EVENT op 0x12 (`Script_EventHandler` 0x1d94c0:194-198 -> `ResetMap_LevelToLoad`, level id read from the stream [M 0x1d94c0:83-93]) inside one of the level's 7 script streams - decode pending. Decode the 7 streams' events [D], find the 0x12 event (or whatever really ends the level [M]), port it through the port's script event handler, and remove the timer. A script 0x12 into 0x0700001b loads the 0x7100018 briefing first [M 0x1d94c0 skipbrief=0, ResetMap_Load 0x1d6360:99-102]: make the port do the same. Pin the end with a real-input test on 07000041 (the exit fires at the game's tick, into the briefing). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-exit-event`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T14:34Z
