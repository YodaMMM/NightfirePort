## Your task: `turret-round-spread`, do turret rounds spread?

The port fires owner -1 turret rounds (wall guns, the Ronin) dead-on; `Bullet_init` 0x12c5a0 has a spread term (and 3 shared-stream draws per shot, with 1 more in GT_Update 0x180498:349) that may apply to them. Settle it from the decompile [M Bullet_init's spread/owner branches, GT_Update's fire path], port the spread and the shared-RNG draws in the game's order if they apply, and pin: a turret burst's hit pattern / draw count (sister: dead-on). Run the gun-turret / ronin binaries, every *0700000a* binary, cross_process_determinism and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/turret-round-spread`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-03T06:29Z
