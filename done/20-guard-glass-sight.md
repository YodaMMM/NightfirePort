## Your task: `guard-glass-sight`, guards see and shoot through glass exactly where the game's do

glass-pierce-2's report (origin/s/glass-pierce-2, docs/reports/glass-pierce-2.md, item 5 of "What's left") leaves the guards' line-of-sight masks against glass untouched: the 08 notes say the guards' aim traces see no panes on master. Read the masks each guard sight and aim trace passes to the line-of-sight call in the decompile (vision, the aim trace, the shot's own ray), with addresses, and which glass materials and kinds each one stops at. Compare the port's guard traces and fix any that differ, in the game's order, keeping the Rand count. List which levels have glass in guards' sight [D]. Pin through real play on 07000008 and 07000001: a guard behind intact glass sees Bond (or does not) exactly as the game's mask says, and shoots when it does. Repair any walk this shifts, gated to its level, never lowering damage or health. Run the ai binaries, glass_pass_every_level_live, dead_pane_live, walkreal_07000001_live, walkreal_07000008_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-glass-sight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-08T09:24Z

---
## Report (done, 2026-11-08T10:52Z)

Closed by the driver: the report is on `s/guard-glass-sight` (docs/reports/guard-glass-sight.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-08T10:53Z)


---
## Report from the worker (done, 2026-11-08T10:53Z)

