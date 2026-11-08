## Your task: `guard-pane-drain-3`, the 07000001 walks stay green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-2, merge current origin/master, and read its docs/reports/guard-pane-drain-2.md plus the 01 ignore messages. Take its two 01 levers: the flank_off re-lure for the d1 real-damage walk and the camper-clear extension for d3, whose ledgers are in the ignore messages. Then the rest of the walkreal sweep (03, 04, 05, 07, 0b, 0c, 0d, 11, 13, 14, 15, 16, 1b, 41 and 4a) one level per tool call, repairing any walk that shifts, gated to its level. The 08 Hind red belongs to the level-08-hind-fight branches, not here. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the bullet binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T18:00Z
