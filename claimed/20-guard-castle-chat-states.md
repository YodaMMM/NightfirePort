## Your task: `guard-castle-chat-states`, the castle's chatting and interrogating guards act as in the game

The guard-noise-attack-states report (origin/s/guard-noise-attack-states, docs/reports/guard-noise-attack-states.md) lists CastleChatGuard1 [M 0x1612e0] and InterogateWalk [M 0x162338] as unported guard states; drone.rs names them only in a comment. Read both state handlers in full, including their message handling, with addresses. Find which on-foot levels' placements use them [D], and what the port does there today. Port them in the game's order, keeping the Rand count, and test them red on the old code. Pin them through real play on a level that uses them: the guards chat or interrogate, and break off when alerted, as the handlers say. Then run every walkreal_*_live for the levels that use them, one level per tool call, and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-castle-chat-states`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T11:14Z
