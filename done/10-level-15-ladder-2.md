## Your task: `level-15-ladder-2`, 07000015: the ladder grab (round 2)

`crates/nf-view/tests/interactables_07000015_live.rs::the_ladder_takes_the_climb` is still `#[ignore]`: the grab cannot fire from any reachable standing spot (the east room's south wall at z -1.95). Find how the game's ladder grab reaches it (the ladder's use sphere / grab volume, the approach the game's floor allows) [M the ladder grab functions].

The previous attempt's findings are in the test's own `#[ignore = "..."]` message on master — read it first; it names the blocker exactly. Real input only (`step_bond`: movement/aim/fire/use; no teleports past the blocker, no channel or state pokes, no debug or cheat paths). Port what the game does at the blocker from the decompile [M] / the disc [D]. If a live number is needed, say exactly which address to read in PCSX2 on a fresh boot, and stop there rather than guess. Un-ignore the test when it passes, with a sister that can fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-15-ladder-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox2 at 2026-10-23T18:52Z

---
## Report (done, 2026-10-23T20:22Z)

Closed by the driver: the report is on `s/level-15-ladder-2` (docs/reports/level-15-ladder-2.md); the worker didn't run finish.sh.
