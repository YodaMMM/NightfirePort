## Your task: `level-13-warm-chain-margin-6`, 07000013's warm-chain real-damage walk gets past door 543 and out the front

Branch from current origin/master and read the ignore message on walk_the_warm_chain_to_the_front_entrance_and_out_real_damage in crates/nf-view/tests/walkreal_07000013_live.rs, plus docs/reports/level-13-warm-chain-margin-5.md on origin/s/level-13-warm-chain-margin-5. The 12 leg's corridor fights carry Bond into door 543's recess, the door opens from east of the hinge and the leaf pins him. Find a route or timing so Bond is clear of the leaf's swing when 543 opens (the game's own door logic is not to be changed for this), then carry the walk out the front entrance. The ignore lists what was tried; don't repeat those. Helpers stay gated to 07000013 and the difficulty, so the operative, agent and plain sisters stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-warm-chain-margin-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-14T01:02Z
