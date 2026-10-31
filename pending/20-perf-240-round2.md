## Your task: `perf-240-round2`, the last frame-time hot spots, behaviour unchanged

Continue `origin/s/wip-perf-240e` (it builds on origin/s/perf-240d; merge `origin/master` first). Target: every level <= 4.17 ms per frame. Left: 07000007's tick frames (~4.3 ms; guards' animation and floor-collision rays in the sim tick) and 0700000a's line-of-sight pass while walking (0.8 ms). The sim must stay bit-deterministic (same results, same tie-breaks: prove it with the determinism/replay tests and a before/after state hash over N ticks on 07 and 0a) and nothing drawn may change (palette + pixel hashes over several views). Measure with frame_path_probe_live.rs. Run frame_cost_live, load_scene_equality, load_scene_digest, every determinism/replay/ai/guard binary and nf-game; give a before/after per-level table. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/perf-240-round2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

