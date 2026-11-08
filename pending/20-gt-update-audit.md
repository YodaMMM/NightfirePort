## Your task: `gt-update-audit`, the 0700000a and 0700000b gun turrets behave as the game's in every state

Two notes disagree. The control-funcs-update audit lists `GT_Update` 0x180498 as unported, quoting the doc comments in crates/nf-game/src/gt.rs:32-33 and drone.rs (the `gt_turrets` field) that call the turret behaviours a known gap. But drone.rs (search `GT_Update 0x180498`) already runs the scripts, the wreck smoke, the deploy and fold rewinds, `GT_Track`'s line of sight and case 7's mounted fire. Walk 0x180498 case by case, plus `GT_Track` 0x17f9c8, `GT_ApplyDamage` 0x1816d8 and the deploy and take-control paths, against the port. Write a table in the report: each case and arm, ported or not, with the port's line. Port what is missing for the single-player turrets (0700000a has six, 0700000b five) and correct the stale comments with what the table shows. Rand draws must match the game's count. Pin through real play on 0a and 0b: a turret deploys when Bond comes in range, tracks him, fires real rounds, takes damage until it is wrecked, and stops. Run the turret binaries (grep tests for `gt_turrets` and `GT_`), walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gt-update-audit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

