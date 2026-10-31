## Your task: `level-09-alarm-switch-walk`, walk to the 07000009 alarm switch instead of placing Bond there

`crates/nf-view/tests/walkreal_07000009_live.rs` `the_alarm_switch_fails_objective_1` (~line 248) sets `sim.bond.pos` beside the ch-124 alarm switch (a teleport, then real E input). Replace the placement with a real walk from the spawn (or from the end of an existing walked leg in the same file) to that switch using the Walker's movement only, then keep the existing aim + E press and the objective-1 fail assertions (ch 0x7C -> mux #278 -> ch 0x61 -> "Alarm Raised" [D mux #278, M 0x1827d8:128-131]). If the route is blocked in the port where it is open in the game, find and fix the cause [M/D] instead of routing around it, and say so. No pokes of state, channels or positions. Run the 09 test binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/09-alarm-switch-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-31T10:11Z
