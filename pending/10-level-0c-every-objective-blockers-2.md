## Your task: `level-0c-every-objective-blockers-2`, land the 0c blocker findings on today's master

`origin/s/level-0c-every-objective-blockers` (its report is on the branch) merged with origin/master (531c08fb, which carries the cable third-person work and the scoped bullet re-query) makes its own `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` RED: "the #283 ride must drop onto the ring SE corner (11.09..11.17), got [35.15, 13.55, -209.95] dropped true" (walkreal_0700000c_live.rs:454). Merge origin/master into the branch, find what moved the #283 ride's drop (what Bond stands on at y 13.55 - a collision face, the drop timing, the wire's end) [D/M], fix the port or re-derive the leg with evidence, and keep the whole walk green. Run the FULL suite on the final tree and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-every-objective-blockers-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

