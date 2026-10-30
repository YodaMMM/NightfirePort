## Your task: `script-event-channel-writes`, act on the script 0xD/0x11 channel-write events

The port parses `Cmd::Event` (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB; `Script_EventHandler` 0x1d94c0's 0xD/0x11 channel-write events are ignored (level-11-every-objective-3 and level-05-every-objective-4 reports, merged: e.g. 07000012 script 06000530 stream 8 [60,60], 06000a25 [137,1]/[137,0], 07000005 06000952 ev 0xD -> ch 0x62). Port both events exactly as the decompile does (which channel table, set vs clear, timing within the frame), with a unit test per event on real disc scripts. Then run every walkreal binary whose levels carry these events (sweep the discs) and report each total; fix or report any walk the new writes change. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/script-event-channel-writes`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T11:39Z
