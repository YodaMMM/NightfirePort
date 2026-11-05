## Your task: `broken-object-bark`, 07000001's guards react out loud when Bond breaks an object

`Break_Kill` 0x12b970 ends with `Drone_BrokenObject` 0x138588, and the port never makes that call: crates/nf-game/src/breakable.rs only documents it. Read work/decomp's 0x138588 in full. It runs only on level 0x7000001, for a player-broken script-type-5 object, and walks the drone list for live type-0x0c drones; the bark is 0x40 + Rand(4) (row 1 of "Left as a table" in docs/reports/audit-voice-triggers.md on origin/s/audit-voice-triggers). Merge that branch in if it is not on master. Plumb the call from the `Breaks` kill path to `Drones`, keeping the game's order of Rand draws, and every gate the decompile shows. Pin it through the real path: Bond shoots a qualifying 07000001 object with a qualifying guard alive and the line plays; the same break on another level, or by a non-player, plays nothing. Run the new test, interactables_07000001_live, walkreal_07000001_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/broken-object-bark`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-05T04:13Z

---
## Report (done, 2026-11-05T05:28Z)

