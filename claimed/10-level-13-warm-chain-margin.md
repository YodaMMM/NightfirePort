## Your task: `level-13-warm-chain-margin`, Chain Reaction can be walked from the warm chain out of the front entrance on real damage

Branch from origin/master. walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage is ignored: on master Bond dies at tick 2448 at (4.03,-368.39,33.35) with 3 hp after a 3.50 hit from drone #5 (weapon 0x19); the same route passed on d09f8b2b3. Find which merge between d09f8b2b3 and master changed the bleed (bisect the walk), say whether the new damage is game-correct [M], and re-route or re-tune the walk (real input only, no top-ups, never lower damage or health) so it passes with a real margin. Un-ignore it. Run the whole walkreal_07000013_live binary and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T10:08Z
