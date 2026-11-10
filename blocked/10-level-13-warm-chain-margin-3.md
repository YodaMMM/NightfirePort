## Your task: `level-13-warm-chain-margin-3`, Chain Reaction's real-damage warm-chain walk green under the game's guard sight

Branch from origin/master (823ff190b: the game's guard sight). origin/s/level-13-warm-chain-margin-2 made walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage green on the old sight (drain_the_wave at the lobby east lane, the spawn-pocket pre-clear at difficulty 3), but it conflicts with the sight batch's own 13 changes (the pre-clears now run for difficulty >= 2 too). Read docs/reports/level-13-warm-chain-margin-2.md on that branch, port its levers onto master's walk, and un-ignore the test. Real input only, never lower damage or health. Run walkreal_07000013_live and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T14:14Z

---
## Report (blocked, 2026-11-10T14:54Z)

