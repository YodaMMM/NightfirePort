## Your task: `level-07-party-door-lock-2`, land the 07 party-door lock tests

Start from `origin/s/level-07-party-door-lock` (merge it first; its report has the chain: door #251 starts open on ch 130, script 0x06000073 stream 8 writes ch 130 <- 0 at the control scene, the door closes and refuses E). Three things block the merge:

1. `party_door_lock_07000007_live` asserts `!channels.on(200)` as permanent game truth ("no writer" is a [D] scan result, not a proof). Keep the lock assertions that come from [M] (the door refuses E after the scene, the free door #244 still opens); turn the ch-200 "never written" claim into a logged observation, or prove it [M — every writer family of a switch channel on this level, including the loadable packs] and cite it.
2. `dump_07000007_doors_and_channel_writers` and `dump_06000073_stream8` are added `#[ignore]`d. Move them to `crates/nf-view/examples/` (probes, not tests) or remove them.
3. Run the tests un-ignored on the tree merged with origin/master; report the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-party-door-lock-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T05:19Z
