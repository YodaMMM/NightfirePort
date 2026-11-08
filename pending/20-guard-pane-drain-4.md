## Your task: `guard-pane-drain-4`, the 07000001 walks stay green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-3, merge current origin/master, and read its docs/reports/guard-pane-drain-3.md plus the 01 d3 ignore message. Take the d3 walk's next lever from that message: ammo at the door phase, then the armed sustained lure. Then run the sweep levels it could not finish (04, 07, 0c, 0d, 13, 14, 15, 16 and 1b) one walkreal_*_live binary per tool call, building with -j2 because -j4 ran the 4 GB sandbox out of memory. Repair any walk that shifts, gated to its level. The 08 Hind red belongs to the level-08 branches. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the bullet binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

