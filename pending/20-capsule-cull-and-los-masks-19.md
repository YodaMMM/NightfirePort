## Your task: `capsule-cull-and-los-masks-19`, the faithful cull + masks without the 13 reds

Same task as `blocked/20-capsule-cull-and-los-masks-18.md` on the queue branch (read it; the previous run ended without a report). Continue `origin/s/capsule-cull-and-los-masks-17` (or -18 if it has commits; merge `origin/master` first). Keep the two disasm-verified capsule rules (the head-on cull on the RAW denom f2 = [sp+128] at 0x10a1f0, +/-0.999 with the 0.5 halves; Dist2Tri's flags 0/99 skip at 0x10a330/0x10a33c). Bisect which part breaks each of the 13 reds listed in the -18 task (the LOS masks on, the head-on cull, the flags-99 skip, the turret muzzle/sphere change), fix what differs from the game with [M]/[D]/fresh-boot [L] evidence, and land only with those 13 and every walkreal_* binary green. Commit and push your branch after EACH bisect step so a reset loses nothing; quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-19`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

