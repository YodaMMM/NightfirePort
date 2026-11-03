## Your task: `level-0d-real-health-crossfire`, walk 0700000d to the C5 door on real damage

`crates/nf-view/tests/walkreal_0700000d_live.rs` `walk_arrival_to_the_c5_door_real_damage` (~line 728) is still #[ignore]d. level-0d-yard-climb-cover (on master, report docs/reports/level-0d-yard-climb-cover.md) proved the perch kill on sniper #31 from cover. After that kill, the climb and wire crossfire still outguns Bond's 100 health: the west roof #27/#28/#29, the dock patroller #33, the south heavy #10, and #16 once the gunfire turns it. Read that report's shooter map and reverse-ray matrix first. Then check each shooter against the game. When does it wake [M the platform-drop trigger and its channel]? Can it see Bond at that range and through the canopy [M the sight/LOS path, D coll-scan]? How often does it hit at that distance on the current difficulty [M the accuracy and damage path, the weapon's table, D]? Fix anything where the port fires, sees or hits harder than the game. Then walk the route a player would take, on real input only: kill order, cover, and timing between shots. Un-ignore the test, and never lower the damage or health to make it pass. Run walkreal_0700000d_live, enemy_accuracy_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-real-health-crossfire`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-03T03:26Z
