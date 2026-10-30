## Your task: `level-13-spawn-to-end-walk`, 13 from the carried load to the mission's end

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out` starts warm. level-11-every-objective-3 (merged) walks 11 -> 12 -> 4A and pins 0x04000035 "Escape from the tower" as announced only: its writer is the whole-mission AND ch 100 [M 0x1827d8:119-123], first set on 07000013 [D 0700004a.md / 07000013 docs]. Add one walk that loads 07000013 the way the game does after 4A (the RamSave carry [M Player_RamLoad 0x1957a8]) and reaches the level's end on real input with every objective set by the game's own writers (no channel pokes), pinning 0x04000035 complete. Fix any port gap it hits with evidence [D/M]. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-spawn-to-end-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

