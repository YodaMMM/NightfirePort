## Your task: `retune-nb-14-5`, 14 infiltration to the delta exit walks green again (round 5)

Branch from current origin/master; if origin/s/retune-nb-14-4 is not on master, merge it in first. Read docs/reports/retune-nb-14-4.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Master changed the guards again (SniperAim ENTER, 07:28), so first re-measure the death on the merged tree. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. The Operative sister walk_the_infiltration_to_the_delta_exit_real_damage was green on origin/s/retune-nb-14: merge that branch in too and keep it green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

