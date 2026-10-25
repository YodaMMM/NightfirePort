## Your task: `level-0a-rain-indoors-4`, the bathhouse shout after the rain fix

Start from `origin/s/level-0a-rain-indoors-3` (merge it first, then origin/master). One red remains, green on master: `bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too` (07000002) — no floor guard's shout voice (ids 111..115) keys within the 4x300-tick window after the plate cycles. The game-true rain consumes more of the shared generator [M Rand_* 0x1e36f0..0x1e3780], which moves the shout's voice pick/timing.

1. Find which draw decides the shout (the voice pick, the alert timing, the guard's reaction) [M], and whether the test pins a game rule or one lucky stream value. Make the test assert the game's rule with a window the decompile justifies (or drive the real trigger the way a player does); never weaken it below [M].
2. Green with rain at the game speed; the recovered reds from rounds 2-3 stay green; run the FULL suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-rain-indoors-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-25T11:47Z
