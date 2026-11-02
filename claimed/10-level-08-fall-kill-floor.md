## Your task: `level-08-fall-kill-floor`, falling off 07000008 ends the mission like the game

`crates/nf-view/tests/interactables_07000008_live.rs` `the_kill_floor_ends_the_fall` (~line 844) is #[ignore]d: after a fall the death NIS holds the body at y 17.56, above the y −24.10 kill floor, so the end-to-end fall never reaches the mission fail. The mid-path is pinned by `hurt_07000008_live`. Find the death NIS's terminal event and what it does in the game (mission fail / restart path) [M: the death-NIS event handler and the fail writer; D the NIS stream], port it so the hold ends the way the game's does, and un-ignore the test. Run interactables_07000008_live, hurt_07000008_live, the death-restart tests and walkreal_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-fall-kill-floor`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-02T04:51Z
