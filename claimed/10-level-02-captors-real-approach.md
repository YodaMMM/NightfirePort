## Your task: `level-02-captors-real-approach`, clear 02's captors without the teleport

`walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit` is green, but its `clear_captors` helper (walkreal_07000002_live.rs ~60-75) stands Bond 2.5 units beside each guard by setting `bond.pos` before the shot ("a straight walk crosses rooms"). Replace that teleport with a real-input approach: walk the nav web to a firing spot with line of sight (or take the shot from where Bond already stands), aim and fire through the real path, the guards live. Fix any port gap it exposes with evidence [M/D]. The walk must stay green with NO bond.pos writes after the spawn. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-captors-real-approach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T15:05Z
