## Your task: `level-16-pit-squad-operative`, the 16 chase completes on Operative with real damage through the pit squad

Branch from origin/master. walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage (Operative) is ignored: the pit squad's ninjas fire at accuracy byte 0 [M 0x14b300:752]; after the chaser at 46.4 Bond dies at tick 15337 to #12 at 0.4 u. First confirm the accuracy byte path (where 0x14b300:752 reads it, what a 0 does in DroneWeap_DoBulletAccuracy 0x179740) so the walk tunes against the game's rule; then build a stand for the pit fight on real input (distance, cover, the level's ammo/armour pickups [D]). Never lower damage or health, no top-ups; helpers gated to 07000016 and difficulty 1. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000016_live and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-16-pit-squad-operative`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
