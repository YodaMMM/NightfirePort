## Your task: `fire-modes-real-press`, launcher, grenade and Samurai fire modes work through the real press

Three tests in crates/nf-view/tests/fire_modes3_live.rs are ignored with harness reasons: `launcher_guided_rides_and_unguided_fires_four` (the 0x2D mode press is refused after the 0x2C camera ride), `mgl_impact_and_delay_through_the_real_mode_press` (the 0x2B leg's trigger edge misses) and `samurai_charge_drains_and_blasts_and_beam_spends_twelve` (the 0x32 draw stalls). Do not assume these are harness faults: the mode press being refused after a camera ride could be what a player gets too. For each, find the true cause against `Player_Weapon` 0x1a24c8 and the mode-change arm, and fix the port if it is wrong, or the drive order if it is not, and say which in the report with addresses. Remove all three ignores. Run the whole fire_modes3_live binary, fix_weapon_fire_modes_live, weapon_matrix_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/fire-modes-real-press`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

