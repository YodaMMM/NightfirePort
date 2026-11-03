## Your task: `level-01-agent-hall-fight-2`, finish 07000001's contact escort on real health at Agent

Continue origin/s/level-01-agent-hall-fight: branch from it, merge origin/master in, and read its report and blocked/10-level-01-agent-hall-fight.md on the queue branch. That session won the hall fight (the hall AND, ch 0x99, fires at tick 4950). Bond now dies at 1.6 health to the last route guards at the stairwell. The report lists two drains. One is the door-phase settle gap: the crossing guard's volley lands between the show and the shot, 76.5 to 42.9 health in the worst run. The other is the route guards' attrition (12/17). Do the report's next move: keep the aim on the crossing guard's live position through the gap. Fetch the pickups on the way before the stairwell, and set the kill order for guards 3, 14 and 5. Make walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` pass. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000001_live, escort_follow_07000001 and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-agent-hall-fight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-03T16:09Z

---
## Report (done, 2026-11-03T16:22Z)

Closed by the driver: the report is on `s/level-01-agent-hall-fight-2` (docs/reports/level-01-agent-hall-fight-2.md); the worker didn't run finish.sh.
