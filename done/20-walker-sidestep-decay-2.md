## Your task: `walker-sidestep-decay-2`, the walker sidestep decay without breaking Night Shift's roof walks

Branch from origin/s/level-0b-dome-wire-sisters (it contains walker-sidestep-decay) and merge origin/master. Its change to the shared crates/nf-view/tests/common/walker_b2.rs turns walkreal_0700000b_live from 9 passed / 0 failed on master to 5 passed / 4 failed. Find which walk behaviour the decay changes on 0b and gate or repair it so 0b stays 9/0 while the 0c and 12 walks it un-ignores stay green. walker_b2.rs is used by 128 test files: run every walkreal_*_live binary one level per tool call before pushing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-walker-sidestep-decay-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T05:54Z

---
## Report (done, 2026-11-12T07:18Z)

