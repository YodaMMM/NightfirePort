## Your task: `glass-pierce`, rounds pass intact glass and hit what is behind it, as the game's do

done/30-dead-pane-collision-2.md (What's left, item 2) records that the game's round passes an intact material-0x10 pane, except weapons with record +0x6c bit 0x800, and continues to the next contact [M 0x12cf44..0x12cf64], while the port's round dies at the pane contact. `Bullet::glass_pass` in crates/nf-game/src/bullet.rs already covers the flight's world trace, so first find which contacts still stop a round on master and read done/30-bullet-pass-water.md and the 11 stairwell bullet-wall reports for what is settled. Read 0x12cf44..0x12cf64 in the disassembly: does the round damage the pane and go on, how much damage carries on, and does it leave an impact on the sheet? Then the guards' side: that report says the guard-fire gate fires through glass where the game's aim trace (mask 0xa) stops on non-0x10 glass. Read that too and match it per material. Census which panes are material 0x10 on each level [D]. If the code cannot settle whether the round leaves an impact at the sheet, list that as a live check for the main agent and do not guess. Pin through real input on two levels: a round through intact 0x10 glass hits a guard behind it, and a fist or laser (bit 0x800) stops. Run dead_pane_live, glass_pass_every_level_live, the bullet binaries, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/glass-pierce`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-07T16:27Z
