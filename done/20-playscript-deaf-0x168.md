## Your task: `playscript-deaf-0x168`, scripted guards on 0x6000168 ignore gunfire while their script plays

`NDrone2_DSTATE_PlayScript` 0x15ba68's ENTER sets `+0x4F8 |= 0x400000` on three scripts: the shared tail of 0x6000168 and 0x6000169 (decomp line 73) and the 0x60007ac arm (line 84). The port's `(PLAY_SCRIPT, msg::SOUND)` arm in drone.rs (around line 7393) tests only 0x6000169 and 0x60007ac, so a 0x6000168 guard breaks out of the script on gunfire where the game's stays deaf [M 0x177800:19-21]. docs/reports/audit-voice-triggers.md row 4 reads this as possibly inverted; the decompile shows both arms set the bit, so 0x168 looks simply missing. Confirm against the disassembly at 0x15bb50-0x15bba0. Read the three clears too (decomp lines 128, 135, 153) and make sure the port's deaf window ends on the same messages for all three scripts. Fix the comment that cites :36-44. List which levels place a guard on 0x6000168 [D]. Pin it on one of those guards with real gunfire, and pin that the guard hears once the script ends. If no placement on disc uses 0x6000168, say so in the report and keep a unit pin. Run cover_nodes_live, fix_06_alarm_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/playscript-deaf-0x168`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-05T05:41Z

---
## Report (done, 2026-11-05T09:34Z)

