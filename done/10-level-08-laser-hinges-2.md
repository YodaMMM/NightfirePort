## Your task: `level-08-laser-hinges-2`, 07000008 can still be walked on 00 Agent with the laser-hinge fix in

Branch from origin/s/level-08-laser-hinges, merge current origin/master, and read its docs/reports/level-08-laser-hinges.md. The fix is green everywhere on 08 except `walk_the_whole_mission_on_real_input_real_damage_00agent`: the burn sparks' stream draws shift the run's timeline, and it dies at tick 25550 at (105.8, 28.37, -119.2) in the lower-level-east leg's return-fire stand on drone #23 (no line of sight for the kill). The stairwell-mouth stand was tried and died at 25829. Re-position that leg under the new stream: find a stand that sees #23 and the pair, or take them in a different order, and re-pin the ledger. The stream itself is the game's and stays. Helpers stay gated to 07000008 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Run the whole walkreal_07000008_live binary, interactables_07000008_live and the laser binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-laser-hinges-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T04:18Z

---
## Report (done, 2026-11-08T05:18Z)

