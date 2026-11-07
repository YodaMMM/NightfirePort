## Your task: `hurt-volume-flight`, the flying hurt volumes on 07000046 follow their path and hurt in flight

`Hurt_Create` 0x17e870 also reads the placement's raw +0x40/+0x42 (u16 count, u16 rate) and +0x44 (path reference) into ext+0x0C, ext+0x04 and ext+0x08, and `Hurt_Update` 0x17e5d0's state 1 flies the volume along that path with `Spline_Interp`, hurting as it goes. docs/reports/unread-params-props.md on origin/s/unread-params-props found the triple non-zero only on 07000046's seven hurt volumes (count 6, rate 0x5A, path words ending 000E/000F/0010) [D], and the port has none of it. First say what these seven volumes are in play on that level, and what starts state 1. Plumb the path reference to the level's path table, port state 1 with the game's rate and end behaviour, and keep crates/nf-game/src/hurt.rs's other states as they are. Pin through the real level: a volume flies its path at the game's pace, Bond in its way takes the damage, and clear of it he does not. Run the hurt binaries (grep tests for `hurt`), the 07000046 binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/hurt-volume-flight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T17:33Z
