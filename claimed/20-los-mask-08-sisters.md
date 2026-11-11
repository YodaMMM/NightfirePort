## Your task: `los-mask-08-sisters`, 07000008 can be walked on Agent and 00 Agent with the game's line-of-sight masks

Branch from origin/s/los-mask-08-walks, merge current origin/master, and read its docs/reports/los-mask-08-walks.md, items 1 and 2 of "What's left", plus the 08 Agent and 00 Agent ignore messages. Agent first: pin the deck pair's enable plate, place the pre-engage where they are visible, then the descent re-tune, in the ignore message's order. Then 00 Agent: the upstream bleed (the walkway pair, the stairwell and descent fights) by the ledger's descent re-tune. The mask rule is the game's and stays. Helpers stay gated to 07000008 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-08-sisters`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T04:46Z
