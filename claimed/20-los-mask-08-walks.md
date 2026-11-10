## Your task: `los-mask-08-walks`, the 07000008, 11, 12 and 14 walks stay green with the game's line-of-sight masks

Branch from origin/s/los-mask-01-walk-2, merge current origin/master, and read its docs/reports/los-mask-01-walk-2.md, items 1 and 2 of "What's left". First the 08 Operative ledger's three levers (the terminal walk-out first), then the Agent and 00 Agent sisters' descent re-tune. Then the 11, 12 and 14 re-time levers written in their ignore messages (mid-floor waypoints and pinned fight sites). One level per tool call, each repair gated to its level and difficulty. The mask rule is the game's and stays. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run walkreal_07000008_live, walkreal_07000011_live, walkreal_07000012_live, walkreal_07000014_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-08-walks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-10T15:54Z
