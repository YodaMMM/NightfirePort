## Your task: `level-01-bonus-floor-walk`, walk onto the 07000001 west-hall Bond bonus and take it

`crates/nf-view/tests/interactables_07000001_live.rs` `the_bond_bonus_walk_over_takes_it` (~line 385) is #[ignore]d: the bonus floor sits ~1.6 below every floor the walker reaches (the spawn-side nav ends on the upper hall at y 7.18; the west corridor's low route rides the escort path). The two bonuses are at #315 (−13.24, z 10.35) and #316 (−26.79, z 14.51) [D]. Find how a player reaches that floor in the game (stairs, a drop, a door, the escort route) [D collision + placements; M any door/lift gate on the way]. If the port blocks a route the game leaves open, fix the port. Then walk there on real input, take the bonus by walking over it, and un-ignore the test. Run interactables_07000001_live and walkreal_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-bonus-floor-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-02T03:51Z
