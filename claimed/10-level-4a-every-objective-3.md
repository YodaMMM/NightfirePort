## Your task: `level-4a-every-objective-3`, the scoped drone bullet flight without breaking the 08 fusebox

`origin/s/level-4a-every-objective-2` (its report is on the branch) makes `walkreal_07000008_live::walk_the_whole_mission_on_real_input` RED, reproducibly (green on master and on master + level-0a-every-objective-4): it fails at "shooting the fusebox wires raises ch 137 [D #219 group-0; M FuseBox_Update 0x1adfe8]". Continue from -2 (merge `origin/master` first). Find why the branch's change (the drone-fired `Bullet::update` querying `segment_bullet_scoped(find_cel(pos))`, the world.rs additions) stops the fusebox-wire shot from landing, and what the game does there [M Bullet_update 0x12e7b0, Collide_Pick 0x1eb320, the fusebox wires' cel/candidate set]. Fix it so both the 4a ride fire leg and the 08 fusebox walk are green. Run the FULL suite on the final tree and quote its total line - no new reds.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-every-objective-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T00:08Z
