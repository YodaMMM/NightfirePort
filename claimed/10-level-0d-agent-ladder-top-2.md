## Your task: `level-0d-agent-ladder-top-2`, aiming freezes the view exactly as the game's does, and 0700000d is walkable on Agent

Branch from origin/s/level-0d-agent-ladder-top, merge current origin/master, and read its docs/reports/level-0d-agent-ladder-top.md plus the Agent ignore message. First its aim-mode audit: read the aim-mode view and cursor update (0x1a4a98 and the pad cursor update) in full with addresses, and settle whether the game freezes the view while the cursor moves in aim mode, and when. If the port differs, fix it in scene.rs's free-aim block in the game's order, pin it through real input on an on-foot level, and re-run every walkreal_*_live one level per tool call, repairing any walk it shifts, gated to its level. Then the walk's levers: the #17 wake lag (delay the second #17 pass 150 to 300 ticks or widen the wake window), then the ladder-top endgame. Helpers stay gated to 0700000d and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run walkreal_0700000d_live, interactables_0700000d_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-agent-ladder-top-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T20:53Z
