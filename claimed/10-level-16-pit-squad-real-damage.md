## Your task: `level-16-pit-squad-real-damage`, the 07000016 pit-squad phase survives on real damage

`crates/nf-view/tests/walkreal_07000016_live.rs` `walk_the_chasers_defeat_completes_the_mission_on_real_damage` (~line 1446) is #[ignore]d: on real damage the pit-squad phase (the 4-squad plus pit shooter #8 at the room's door) kills Bond at ~48 health. The route, both shelters, the chaser's range-keeping kill and pulse 2 already survive. First check the squad's damage against the game (their weapon, hit damage, fire rate and accuracy at that range [M the weapon/accuracy tables; D their placements]) — if the port deals more than the game, fix the port. Then give the walker honest cover tactics for that phase (cover the room offers [D collision], shooting from it) so the walk survives without health top-ups, and un-ignore the test. No pokes of health or state. Run walkreal_07000016_live with --test-threads=1 and quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-pit-squad-real-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-02T04:22Z
