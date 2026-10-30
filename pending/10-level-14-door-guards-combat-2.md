## Your task: `level-14-door-guards-combat-2`, finish the 14 door-guard fix

Continue `origin/s/level-14-door-guards-combat` (merge `origin/master` first; read its blocked report). It narrowed the port's divergence to two things: (1) the walker #25's route ARRIVES 1.5 units short of MissionPath 5's end because `Route::follow` uses radius 2.0 with a 2.0 lookahead creep (nav.rs ~566/719/775), so ENEMY_MISSION's arrival arm (route code 3 -> ChangeToAttackMode) fires — the game's guards stop AT their route ends and never get code 3 [L; M 0x15b110:19-40 arrival compare `route+0x8C <= AINetwork_GetRouteDistance`, 0x156320]; pin route+0x8C's writer and the game's threshold, and fix the port's arrival; (2) the clones #23/#24 (templates #778/#780, group 9) spawn at alert 1.0 and go straight to 0x5a — find how the game folds a spawn at alert 1.0 [M 0x14cbd8:122-155 the +0x5A2 fold; D the template blocks]. Turn `door_guards_hold_07000014_live` into real pins: no drone in 0x81 or 0x5a over 3000 ticks, the two 0x1b guards stopping at (-51.6, 1.0, 41.2) / (-51.2, 1.0, 38.8) within 0.5. Run every *guard*/*drone*/walkreal binary the route change can touch; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-door-guards-combat-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

