## Your task: `level-14-kiko-corridor-2`, 07000014's Operative real-damage walk gets past the Kiko corridor mouth

Branch from origin/s/level-14-kiko-corridor, merge current origin/master, and read its docs/reports/level-14-kiko-corridor.md, "What's left", plus the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage. The corridor door now opens; the walk dies to the west-hunting guards at/behind the corridor mouth near (-58.92, 0.73, 37.67) around tick 43690. Take the ignore message's levers in order: a second clear_hostiles pass after the corridor waypoint, then the fight-quiet window over follow_nav_route's first legs, then the resupply/loot pass on the corridor floor before re-engaging, judging each over several real runs. Helpers stay gated to 07000014 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-kiko-corridor-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T16:00Z
