## Your task: `real-damage-routes-08-11-14-03`, real-damage walks: finish 03, 08, 11 and 14

On origin/master (after origin/s/enemy-stats-3 merges) the real-damage walk variants (walker return fire, `NF_WALK_NO_TOPUP`, see crates/nf-view/tests/common/damage_log.rs) survive but the ROUTE then breaks on: 03 (captors killed before the servant rescue), 08 (wedges on the walkway leg), 11 (a stair-height check misses by 0.01), 14 (misses the stairwell jump). Fix each route on the walker side only (never game code or an assertion — if a check misses because the game differs, prove it with an address) and add real-damage variant tests that complete each level. Run every walkreal binary for 03/08/11/14 and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-damage-routes-08-11-14-03`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

