## Your task: `guard-radii-16-chasers`, the 16 chasers walks under the game's guard radii

Branch from `origin/s/guard-radii-finish`, merging `origin/master` first, and push to `s/guard-radii-16-chasers`. Read item 6 of docs/reports/guard-radii-finish.md on that branch. Two walkreal_07000016_live tests are red: `the_chasers_punch_lands_the_record_damage` (Bond bleeds to 17.4 by tick 4215 and dies at 8519 to drone #9, 0x500001e, full hp) and `walk_the_chasers_defeat_completes_the_mission_on_real_damage` (dies at 10959 at (-5.83, -21.99, -163.14), last hit -10.00 from the world). Both are chip damage from pistol chasers #6, #7 and #9, who stand closer on the game's radii. Take the levers it names: kill those three in the early phases while they are exposed at 4.6-13 u, and route the defeat walk's end around the hurt volume at (-5.8, -163); identify that volume from the placements [D] first. The radii are the game's: never put 2.0 back, never lower damage or health. Test-side only, helpers opt-in. Keep the d2/d3 sisters' measured ignores unless they go green. Run walkreal_07000016_live whole and quote it, then nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-radii-16-chasers`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-06T03:34Z
