## Your task: `level-0a-office-hatch-3`, the Night Shift office walk re-checked on today's master

`walkreal_0700000a_live`'s office tests are ignored (read each ignore message: the wall stopper, the settle heights, the doorway glide). Master has changed since (the head-on capsule cull and LOS masks are queued separately; the placement-matrix and hand-back rules landed). Re-run the ignored walks on master, find each current exact blocker, and fix it from the game's behaviour [M Player_Collision and the walk/settle path; L the 2026-10-26 heights in the ignore messages] - real input only. Un-ignore what turns green; re-ignore the rest on their exact blockers with evidence. Do not change motion feel (look/interp/blend) without an [L] measurement. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

