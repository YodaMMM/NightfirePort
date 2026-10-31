## Your task: `level-0a-ignored-checks`, 0a: the two ignored position checks

`walkreal_0700000a_live` has two #[ignore]d tests (a doorway's walking height and a corner crossing). Read their ignore notes and the merged capsule work's notes. Find what the game does at each spot [D collision; M the capsule contact rules — head-on cull on the raw denom 0x10a1f0-0x10a2e8, Dist2Tri flags 0/99 0x10a330 — and Player_FeetOnPoint 0x194188], fix the port gap with evidence or re-derive the expectation where it was port-derived, and un-ignore what can be un-ignored; name the live check for anything left. Run walkreal_0700000a_live and interactables_0700000a_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-ignored-checks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

