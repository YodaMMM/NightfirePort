## Your task: `level-13-guards-start-alerted-9`, Phoenix Fire Part 4: guards still start alerted in play

Earlier tasks (guards-start-alerted-1..8, merged) pinned the vision bit per the game, but in real play on 07000013 the guards still start alerted: moving in quietly, they are already shouting and attacking. Reproduce it on the REAL path first: a fresh 07000013 load reached the way a player reaches it (the 0700004A lift arrival / the level carry, not a bare load) and the player's first seconds of quiet movement - log each guard's state, alert level, vision bit and what first sets them alert (the message, the channel, the script). Compare with the game [M the drone alert writers; L a fresh-boot PCSX2 trace if needed]. Fix the difference, and pin it with a real-input test from the level start: the guards stay unaware until they see or hear Bond. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T16:46Z
