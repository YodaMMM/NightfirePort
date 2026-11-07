## Your task: `real-health-00agent-03-7`, 07000003 can be walked to the end on 00 Agent with real health

Branch from origin/s/real-health-00agent-03-6, merge current origin/master, and read that branch's docs/reports/real-health-00agent-03-6.md plus the ignore message on `walk_the_full_route_and_complete_every_objective_real_damage_00agent`. The walk now dies at t410740. Take its next lever: a longer aim-None tolerance on the close-hitter unseen stand (the #3 fights need about 150+ ticks to round the corner and land on hp 18), then consider widening the close-hitter radius past 8 u for #9's 10.3 door-gap wedge, then read the new ledger. Keep `close_hitter_reengage` and `dry_fight_resupply`. Do not repeat the runs that report lists as reverted, and standing pre-emption is already proven worse. Every new flag is opt-in and defaults off, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

