## Your task: `mine-shield-by-guard`, a guard in front of a mine takes the round

On origin/master the player-round pass tests 0xFE mines (`Mine::segment_t`) before drones, so a guard standing between Bond and a mine does not block the round as in the game, where the round takes the NEAREST hit along its path [M Bullet_update 0x12e7b0 -> Collide_Pick 0x1eb320 / Collide_PickObj 0x1eae80, the first-hit rule; Bullet_CollisionHandler 0x12ce80]. Make mines, drones, stuck tripbombs and world geometry compete by hit distance in `update_player_bullets` (crates/nf-game/src/drone.rs), and pin: a guard in front shields the mine (sister: step the guard aside and the same round detonates it). Run the mine/tripbomb tests, every *07000015* binary, ai_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mine-shield-by-guard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

